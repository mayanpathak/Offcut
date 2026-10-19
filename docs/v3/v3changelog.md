# v3changelog.md - record of every change made while building V3

**What this is.** A record of every change made while building V3: what was done, what differs from the plan, the guide or the prompts, how it was checked, and what is still open. `v3implementation.md` says what each file contains; `v3buildguide.md` gives the order of work; `coding-promptsv3.md` holds the prompts (61 to 90).

**How it is kept.** One entry per prompt, or per change made outside a prompt. Entries are in date order, oldest first; new entries go at the end. Every commit that changes the repository carries its entry. The two tables "Open items" and "Known issues" are rewritten when an item opens or closes.

**References.** `§n` is a section of `v3implementation.md`, `G n.m` a step of `v3buildguide.md`, `D-n` a decision of §2, `TS §n` of `technicalspec.md`, `V2 §n` of `v2implementation.md`, `V2 CL item n` an open item or known issue of `v2changelog.md`.

**The baseline.** V3 starts from `origin/main` at `8a271a9967ab1e80826feec1107b307da45b80b5`, the deployed V2 (tagged `v2`). Every frozen-file check compares with that commit. The counts there (Entry 1): 262 Rust tests passed with 2 ignored, 92 Vitest, 12 Playwright (`pnpm e2e`), 30 media cases (`pnpm e2e:media`). No count may fall.

---

## Open items
| # | Item | Owner | Version |
|---|---|---|---|
| 6 | The `buildplan.md` half of V2 §27's copy-back list (the schedule moves of D-18, D-20, D-21, D-23, D-24, D-33) is not applied. Prompt 61 may change `technicalspec.md` only | Agent | Prompt 89 (copy-back) |
| 7 | **The Linux ffmpeg URL is short-lived.** BtbN deletes its dated autobuild releases after about six weeks (on 2026-10-16 the oldest one listed was of 2026-08-31), so the URL of Entry 1 stops working around the end of November 2026. Prompt 87 writes it into `ci.yml`: check it then, and choose there between a newer build of the same day and a copy kept on the asset host | Agent | Prompt 87 |
| 15 | **`CLOCK_WORDS` holds `a.m` and `p.m` since Prompt 77**, the forms `tokenize` gives "a.m." and "p.m." (they never equalled `a.m.` / `p.m.`). Copy back into §9.1's lexicon table | Agent | Prompt 89 |
| 17 | **Prompt 77 as built** (its entry): `Candidate` gains `sentence` (§9.3's struct; `resolve` keeps one KeywordPop per sentence and has no transcript); `lib.rs` step 5 takes a score within `SCORE_NOISE` (1e-6) of a whole hundredth as that hundredth before the threshold (0.65 + 0.20 is 0.84999996 in `f32`: no marker-only list could reach 0.85); the X1-X5 bounds and `VERSION_DOTS` are named constants of `config.rs`. Copy back into §9.2 step 5, §9.3, §9.7, Appendix D, and §23 item 27 | Agent | Prompt 89 |
| 20 | **The split decimal point, as built 2026-10-18** (entry "Outside a prompt: the split decimal point"): `offcut-text`'s `with_split_groups` also joins a word of a point and digits only (`4` `.7`) to the whole number before it. Copy back into v3implementation §8 (beside D-42's split groups, with the founder's two cases) and TS §17.2 | Agent | Prompt 89 |
| 18 | **`redetect_sentence` as built** (Prompt 78's entry): the scope also takes the sentences of every event of `prev` that reaches into it, repeated until none crosses its edge (D-86 and §9.2 step 1 take only the events that touch `s`; an event crossing the 45 s edge was then neither kept nor detected again, and was lost). §19.13's fixed-event row is played on the FromTo of sentence 5 and an amount in sentence 6: an event after `s` is always inside the scope, and no sentence after `3,000 hours` meets its window under synthesized timing. Copy back into D-86, §9.2 step 1 and §19.13 | Agent | Prompt 89 |
| 14 | **An edited number keeps its id, decided 2026-10-18.** §9.9, §19.11's edit row and §19.13 of the plan and Prompts 77 and 78 now say so (Prompt 76's entry). Left: TS §17.6's edit bullet ("a new id is created; its toggle state is not inherited"), listed in §23.2's copy-back | Agent | Prompt 89 |
| 16 | **§9.4 and §9.5 as built** (Prompt 76's entry): "the same explicit unit" read as the same `Unit` variant from the quantities' own words, `None` and `None` included (so "from 2019 to 2024" scores the plan's 1.00); a list announcer of any count claims the markers that follow it, so "eight reasons" leaves no marker-only list; an item's words include numbers in digits; a list's score is multiplied by the least confidence of its anchor words (TS §17.4). Copy back into §9.4, §9.5 and D-105, D-107 | Agent | Prompt 89 |
| 13 | **§8.6 and §9.9 miscount the V2 tests V3 changes** (each says one): eleven `offcut-text` assertions and one `offcut-detect` test asserted V2's stand-ins and were set to V3 values (Prompt 73's table). Copy back into §8.6, §9.9 and G 6.4, and correct the prompt's "exactly one" check | Agent | Prompt 89 |
| 12 | **§7.9's "best τ*" halves the pitch of a periodic voice at random**: `prosody.rs` takes the first autocorrelation peak within 0.9 of the best (`FIRST_PEAK_SHARE`, Prompt 68). Copy back into v3implementation §7.9 and Appendix D | Agent | Prompt 89 |
| 11 | **§19.4's pink-noise row contradicts P4**: pink noise at -40 dBFS RMS reads -40.1 LUFS, beyond `MAX_GAIN_DB` of the target, so P4's capped-gain exception applies; `loudness_target.rs` asserts P4 (Prompt 67). Copy back into v3implementation §19.4: that expectation, or a level of -35 dBFS RMS | Agent | Prompt 89 |
| 10 | **`pnpm fixtures` takes 118-166 s on D1**, not the "under a minute" D-94 and §17.1 assume (40 s); on the Linux runner, where it runs since Prompt 67, it takes 58 s. The two 1080p 60 s clips take most of it even with 4 x264 threads (Entry of Prompt 63). It runs on every `ci` run and every `e2e-media` run: Prompt 87 reads the Linux runner's time and decides between paying it and caching `fixtures/.gen/` by the `STAMP` key (D-94 says no cache) | Agent | Prompt 87 |

## Known issues
| # | Issue | Where | Version |
|---|---|---|---|
| 4 | **Export pressed while the preview is still attaching throws `IllegalTransitionError` (`preview: "attach" is not allowed in state "locked"`).** `control-preview.ts::attach` awaits the render worker's `attachPreview`; an export that starts in that moment locks the preview (`lockForExport`), and the late `previewStore.attached()` is refused by the preview machine. Seen in a dev build on 2026-10-18 when a test pressed Export the moment the canvas appeared; a person would have to press it within the attach's few milliseconds. The export itself completes; the preview may need "start over" to play again. V2 code, frozen in V3 (`usecases/control-preview.ts`, `state/**`); V2's E2E waits for Play first, so no suite meets it | `web/src/usecases/control-preview.ts`, `state/preview-store.ts` | V4 |
| 1 | `nnnoiseless` 0.5.2 depends on `easyfft`, which keeps a process-wide cache of FFT plans (`generic_singleton`, `parking_lot`). The purity check of §3.2 passes (none of the six forbidden crates), and it is not code of a pure crate, but it is global state inside `offcut-dsp`'s tree. Its output does not depend on the cache; `determinism`-style checks in Prompt 67 should still run the chain twice in one process | `crates/offcut-dsp` (Prompt 67) | V3 |
| 3 | **`pnpm e2e` on D1 depends on the speed of D1's internet connection.** `landing.spec.ts` (V2) loads the landing page against the real asset host: `page.goto` waits for the page's `load`, and the demo video (10,376,359 bytes) is fetched from the asset host. On 2026-10-17 that download took 3 s in the morning and 18 to 91 s in the afternoon; four to six landing cases then time out at 30 s with the page fully drawn. CI's runner (step "Playwright, the non-media suites" of `ci.yml`, the same `pnpm e2e`) is not affected. A slow-network run on D1 is read from CI instead, and the entry says so. A fix (routing the asset host in that spec, as the media suites do) changes a V2 test and is not made in V3 | `web/tests-e2e/landing.spec.ts` | V4 or later |
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
| Founder's decisions on §23 items 26 and 27 | item 27, 2026-10-16: written emphasis gets `pitch_z = 2.0` (score 0.90); no rounding before the threshold (Prompt 62). Item 26, 2026-10-18: `clear()` per repo at import step 4, applied in Prompt 80 (open item 3; Prompt 76's entry) |

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

---

## 2026-10-17 - Prompt 65 (V3 05/30): labeled set part 2, a01, a02, README; labels frozen

**Done.**

- **The founder's words for `v3testclip.mp4`** were given in the prompt's message on 2026-10-17, as a transcript with timestamps. Written, without the timestamps, sentence by sentence into `~/v3testclip-words.txt` (outside the repository, 12 sentences) and into the README (below). Open item 2 is closed.
- **`w16-open-source` to `w29-morning-routine`**, the rows `w16`-`w29` of §17.5's file plan, as in Prompt 64: sentences first, then the labels by what a person would expect. `w23` hides word 28 ("thick", inside the second list item: `{ "28": "" }`) and also announces four fixes with three markers, so it is the set's partial list and its hidden word; it expects nothing. `w26`'s two phone-like runs are made up (`5550147`, `5550193`).
- **`a01-scriptA.json`**: the 17 sentences of "The V2 reference clip as recognized", word for word: 157 words, `3000` at word 23, `$12` and `,000` at words 132-133, no `*` marks. Expected: NumberReveal `3000 hours` (3,000 hours, label "editing by hand"), FromTo `from 90 minutes to 10 minutes`, ListReveal `three reasons …` (count 3, "3 REASONS"), NumberReveal `$12 ,000` (12,000 USD, label "in sales"); no KeywordPop. Tags `from_frame`, `announcer`, `X5` (the misheard `8`).
- **`a02-v3testclip.json`**: `source: "asr"`, `author: "founder"`, `sentences`, `words`, `prosody` null. Expected from the founder's words: NumberReveal "500 paying users" (500, `count:users`), NumberReveal "500 paying customers" (500, `count:customers`), NumberReveal "$2,000" (2,000 USD, label "in revenue"). "three months" and the range "three to four months" are no events (tag `no_change_pair` for the range); "The first plan" is not a list. `notes` as the prompt words them, and that a `say` the recognizer writes differently is a change of `say`, not of the label.
- **`fixtures/speech/README.md`** (§17.7), additive: the five TS clips marked "derived in V3 from the reference clip (D-76)" with each one's cut, and that they hold the reference clip's words, not Scripts B, C and D; a row for `testclips/v3testclip.mp4` (D-99, D-100); after V2's expected feed, "Expected events of V3" (the five events, with word ranges, the KeywordPop conditional on measured prosody) and "Expected feed of V3"; two new sections, "The derived clips as recognized" (filled at S8, Prompt 82) and "The V3 test clip as recognized" (the founder's twelve sentences now, the three expected events, the recognized transcript at S8). Every V2 line is kept; the five table rows gain text in their last cell.
- **Coverage, by the G 2.6 checker.** `w01`-`w29`: `29 files, 8 without events, 0 problems`; kinds NumberReveal 10, ListReveal 7, FromTo 11, KeywordPop 9; tags `X1` 2, `X2` 6, `X3` 2, `X4` 2, `X5` 5, `announcer` 4, `marker_only` 3, `from_frame` 5, `change_verb` 6, `no_change_pair` 2, `edit_number` 1, `hidden_word` 1, `partial_list` 1, `years_from_to` 1. All 31: `31 files, 8 without events, 0 problems`; NumberReveal 15, ListReveal 8, FromTo 12, KeywordPop 9, `announcer` 5, `from_frame` 6. Every minimum of §17.5's coverage table is met; Requirement 2 of the prompt holds exactly.

**Differs from the prompt or the plan.**

- **`w25` also holds "from 2019 to 2025"** (tag `years_from_to`). §17.5's coverage table requires "years in a from-to" as an edge case, and no row of the file plan names it; the conference-trip row is about dates and allows it (the prompt's error handling: add a missing case to the file whose row allows it). It changes no kind total.
- **`w19`'s two quantities are "about 20 minutes on captions and $15 a month on music"**, not the plan's example "$5,000 on gear and 20 hours of setup": the row expects no event, and a person would expect $5,000 to be animated. `notes` says so.
- `a01`'s `author` is `"founder"`: the words are the founder's recording; the labels follow the README's expected events.
- The founder's words hold the founder's name. They are committed as the founder's own words (D-112, confirmed at Prompt 90).

**Checked.** Every file parses; the checker's totals above; `a01` has 157 words with words 132-133 `$12` `,000`; the README's word ranges were computed from `a01` (`3000 hours` 23-24, `brutal` 31, the FromTo 54-59, the list 75-94). `rm -rf fixtures/.gen && pnpm fixtures && pnpm fixtures`: "generated 25 fixtures", then "fixtures are up to date". `git ls-files crates/offcut-detect/src` holds none of `from_to list keyword exclusions resolve`; no private path in `git status`. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `cargo test --workspace` 262 passed, 0 failed, 2 ignored, Vitest 92, `pnpm build` exit 0 (WASM sizes unchanged), `pnpm e2e` 12 passed (1.5 min); the frozen check and the tracked-private check print nothing. Counts equal the baseline.

**Frozen.** All 31 labeled files are frozen from this commit on (D-87): a label changes only with a line here naming the file, the label and the reason, and "the detector disagrees" is not a reason. `a02`'s `sentences`, `words` and `prosody` are filled at S12 (Prompt 88) as data. The commit's SHA is recorded in Prompt 66's entry as "labels frozen at <SHA> on 2026-10-17" (G 2.8).

**Found, not fixed.** None.

---

## 2026-10-17 - Prompt 66 (V3 06/30): DSP constants, high-pass, compressor, loudness, limiter

**Labels frozen at `9c96667` on 2026-10-17** (G 2.8): the commit of Prompt 65 holds all 31 labeled files; `crates/offcut-detect/src/` held V2's four files only (`config.rs event_id.rs lib.rs number.rs`). CI on `9c96667`: `ci` and `e2e-media` success.

**Done.**

- **`crates/offcut-dsp/src/config.rs`** [ONLY DSP constants]: the 32 `pub const` of §7.2 verbatim (23 chain, 9 prosody), and `ChainConfig` with the seven fields of TS §34.3 and `Default` (`denoise: true`). `Lufs::new` and `Dbfs::new` return `Option`, and neither type has a `Default`; the two helpers `lufs(v)` and `dbfs(v)` return `Lufs::new(v).unwrap_or_else(|| lufs(0.0))`, which always ends because 0.0 is finite: no `unwrap`, no `expect`, no `panic!` (§7.2's "const helper" pattern; they are `pub(crate)` because `loudness.rs` uses `dbfs`). One more value lives here, `pub(crate) const NO_PEAK_DBFS: f32 = -200.0` (§7.6's "-inf is reported as -200"), so that no DSP number is written anywhere else; it is not one of the 32, which the prompt's `grep '^pub const'` still counts as 32.
- **`highpass.rs`**: RBJ cookbook high-pass, Q = `HPF_Q`, Direct Form II transposed, `f64` coefficients and state; a cutoff of 0 or not below `rate / 2` passes the signal unchanged.
- **`compressor.rs`**: the gain computer of §7.5 (soft knee in the log domain), a peak follower in dB with `exp(-1 / (0.005 x rate))` attack and `exp(-1 / (0.080 x rate))` release, gain `min(1, 10^((out - level) / 20))`; no look-ahead, no make-up gain. The follower starts at the level of a zero sample, `20 log10(1e-9)` = -180 dB, computed by the same `level_db` the samples use rather than written as a second constant.
- **`loudness.rs`**: `integrated` (`ebur128` 1 channel at 48 kHz, mode I, channel 0 set to `Center`, because `ebur128`'s default map for one channel is `Unused`; fed in `BLOCK_SAMPLES` chunks; a `-inf` or any error gives `None`), `sample_peak`, `gain_db`, `apply_gain`, as §7.6.
- **`limiter.rs`**: `limit(samples, ceiling, rate) -> usize` as §7.7. The moving minimum over `[n, n + 240]` is one backward pass with a monotonic queue; the moving average is a running `f64` sum in a fixed order (P8). The required gain, the attack gain, the release and the product are computed in `f64` against the `f32` ceiling widened to `f64`, so that a sample brought exactly to the ceiling cannot round above it in `f32` and the guard clamp touches nothing. The release is written `g = 1 - (1 - g_prev) x release`, so a gain of exactly 1 stays exactly 1 and a signal under the ceiling is unchanged bit for bit. Before the first sample the look-back window takes the least required gain over `[0, k + 240]`, so a peak in the first 240 samples is ramped to like any other.
- **`lib.rs`**: `pub mod compressor, config, highpass, limiter, loudness` beside V2's `resample`; `pub use config::ChainConfig`. `chain`, `denoise` and `prosody` arrive in Prompts 67 and 68.

**Differs from the prompt or the plan.**

- `NO_PEAK_DBFS` in `config.rs` (above): §7.6 gives the value; the prompt counts 32 constants, and this one is a reporting value, not a tuning constant.
- The limiter's spike case asserts recovery to within 1% of the input 10,000 samples after the spike, not exactly: with a 50 ms release (2,400 samples) the gain is still 0.2% below 1 there, which is the release §7.7 specifies.
- `config.rs` has the constants' comments as line comments after each value, as §7.2 prints them; `rustfmt` keeps them.

**Checked.** `cargo test -p offcut-dsp --lib`: 24 passed: `config` 1, `highpass` 6, `compressor` 4, `loudness` 5, `limiter` 4, and V2's 4 `resample` cases; every limiter case asserts 0 clamped samples. `grep -cE '^pub const [A-Z_0-9]+: ' config.rs`: 32. `cargo clippy -p offcut-dsp --all-targets -- -D warnings`: clean. `cargo build -p offcut-dsp --target wasm32-unknown-unknown --release`: finished. Failure-path drill: `let x = v[0];` in `highpass.rs` made clippy fail with `indexing may panic`; removed. `grep -niE 'silence|pause|gap'` over `crates/offcut-dsp/src`: `ABSOLUTE_SILENCE_LUFS` and `absolute_silence`, the comments saying the crate cuts nothing, V2's resampler comments, a test name and the compressor's starting level: nothing that detects or classifies silence. Files: 105 to 163 lines. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `cargo test --workspace` **282 passed** (262 + 20), 0 failed, 2 ignored, Vitest 92, `pnpm build` exit 0 (WASM sizes unchanged: no code uses the new modules yet); the frozen check and the tracked-private check print nothing. **`pnpm e2e` failed on D1 three times** (four to six `landing.spec.ts` cases timing out at 30 s) **while the asset host was slow from D1** (the demo video took 91 s, then 18 s, against 3 s in the morning): known issue 3. This prompt changes nothing under `web/`. The `e2e` reading of this gate is CI's `ci` job on this commit (its step "Playwright, the non-media suites" runs the same `pnpm e2e`), read before Prompt 67: **success** on `2d21c2d` (`ci`, whose `pnpm e2e` passed on the runner, and `e2e-media`), filled in with Prompt 67.

**Found, not fixed.** None.

---

## 2026-10-17 - Prompt 67 (V3 07/30): denoise, run_chain, P1-P8 tests, measured denoiser delay

**Done.**

- **`crates/offcut-dsp/src/denoise.rs`** (§7.4): `Denoise::new(mix)`, `process(input, out)`, `finish(out)`. Input collects into 480-sample frames, scaled by `DENOISE_SCALE` into `nnnoiseless::DenoiseState::process_frame` and back; each wet sample is mixed with the dry input delayed by `DENOISE_DELAY_SAMPLES` through a `VecDeque` ring (`wet x mix + dry x (1 - mix)`); the first `DENOISE_DELAY_SAMPLES` mixed samples are dropped. `finish` records how many outputs are owed (the samples consumed) and feeds zeros, which are not counted as input, until that many are appended; a frame's surplus beyond it is not appended. The VAD value is bound to `_voice_activity` and not read. Inline: total length over many calls and `finish` for 1, 479, 480, 481, 1,000 and 48,001 samples; silence stays under 1e-6; `mix` 0.0 gives the input exactly.
- **`chain.rs`** (§7.8, TS §34.3): `Flow`, `ChainOutput`, `run_chain`, steps 1-7 as the plan's table. With `cfg.denoise`, the denoiser's output vector `den` (capacity = the input's length) is filled block by block, the compressor runs on the samples it gained in that block (`den.get_mut(from..)`, no slice indexing), and after `finish` `den` becomes `work`: at most three clip-length buffers (`pcm48` borrowed, `work`, `den`). Without, the compressor runs on the high-passed block in place. Step 6 corrects once at most, only when the first measurement passed the gate and the gain was not capped; `loudness` is the last measurement or `cfg.absolute_silence`. Two `debug_assert_eq!` on the length; no pad or trim branch anywhere. Inline: 1 sample; 48,001 samples; `Cancel` at the first call (`Ok(None)`, one call); `denoise: false` (same length, -14 ± 0.5 LUFS).
- **`lib.rs`**: `pub mod chain`, `pub mod denoise`, `pub use chain::{ChainOutput, Flow, run_chain}`; `DspError`'s comment now says what returns it.
- **The fixture helper** of §6.4, the same 25 lines in each of the three test files: the directory from `OFFCUT_FIXTURES_DIR` or `<CARGO_MANIFEST_DIR>/../../fixtures/.gen`; `STAMP` must hold the SHA-256 (`sha2`) of `fixtures/manifest.json` as its `manifest_sha256`; any mismatch or missing file panics "the generated fixtures are stale or missing: run `pnpm fixtures`". PCM is read with `as_chunks::<4>()` and `f32::from_le_bytes` (clippy refuses `chunks_exact` with a constant size).
- **`tests/chain_no_clicks.rs`** (§19.5): 7 cases and `chain_timing` (`#[ignore]`). The click check counts the boundaries it checked: 5 for the 5.5 s sine, 3 for the 3.2 s noise, 59 for the 60 s portrait speech.
- **`tests/duration_preserved.rs`** (§19.6): the 7 cases with TS §34.3's names. `prop_length_equals_input` draws half of its cases from 1..=300,000 and half from the edge lengths 1, 2, 479, 480, 481, 959, 961, 47,999, 48,000, 48,001, 96,001, 32 cases per run.
- **`tests/loudness_target.rs`** (§19.4): 10 cases. The guard-clamp case reruns the chain's steps 3-6 with the crate's public stages (`run_chain` does not return `limit`'s count) over eight signals, and asserts 0 for each.
- **`DENOISE_DELAY_SAMPLES` measured: 480** (one RNNoise frame). `impulses_stay_aligned` prints the offset of every impulse for the denoiser alone at mix 1.0 and for the whole chain: all 19 are 0 at 480; at 0 (drill 1) all 19 are +480. The constant's comment is "measured 2026-10-17 by impulses_stay_aligned: one RNNoise frame"; `docs/v3/experiments.md` holds the reading. Denoise stays on.
- **Native timing** (`chain_timing`, release build, D1): 0.607 to 0.617 s for the 60 s portrait PCM (three runs), about 4.7 million samples per second. TE-13 is read in Prompt 68.
- **CI (brought forward from Prompt 87 at the founder's agreement, so that `ci` stays green):** `ci.yml` gains `FFMPEG_LINUX_URL` and `FFMPEG_LINUX_SHA256` (Entry 1's archive) in `env`; "Install the build tools" fetches the archive through the job's own checksum-checking `fetch`, installs `ffmpeg` and `ffprobe` and prints the version line; two steps before `cargo test`: "Restore the reference clip" (`actions/cache`, the same pinned action as the job's other caches, path `fixtures/.cache/media`, key from `web/src/net/asset-fetch.ts`) and "Generate the fixtures" (`pnpm fixtures`, which downloads the clip from `VITE_ASSET_BASE_URL` on a cache miss and checks its hash). The Windows job (`e2e-media.yml`) runs no Rust test and is not changed: open item 9 stays with Prompt 87. Prompt 87 still writes the rest of D-94 (`e2e-media.yml`, `check-external-facts.mjs`).
- **Prompt 66's entry**: its gate's `e2e` reading, left open there, is filled in: CI on `2d21c2d` passed.

**Differs from the prompt or the plan.**

- **`pink_noise_reaches_the_target` asserts P4's exception, not the target.** §19.4 says pink noise at -40 dBFS RMS reaches -14 LUFS ± 0.5 ("Same"). Measured: that signal reads -40.1 LUFS at the input, 26.1 LU below the target, more than `MAX_GAIN_DB` (24) before the denoiser takes anything (after high-pass, denoise and compressor it reads -45.2 LUFS), so the gain is capped and the output reads -21.2 LUFS: exactly what P4 states ("unless the gain was capped"). The case keeps the plan's signal and level and asserts P4 as written: the gain was capped, and the output reads what the loudness step was given plus 24 dB (± 0.5), peak at most -1 dBFS, every sample finite. To copy back into §19.4 at Prompt 89: either this expectation, or a level of -35 dBFS RMS for an on-target case.
- `chain_timing` carries `#[allow(clippy::disallowed_methods)]`: `clippy.toml` forbids `Instant::now` in every target, and this ignored test exists to time the chain. The library reads no clock.
- The three test files start with `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]`, as V2's `mux_roundtrip.rs` does.
- `prop_length_equals_input` runs 32 cases per run (half at the edge lengths), not proptest's default 256: a case is up to 300,000 samples through RNNoise.

**Checked.** `cargo test -p offcut-dsp`: lib 31 passed; `chain_no_clicks` 7 passed, 1 ignored; `duration_preserved` 7 passed; `loudness_target` 10 passed (0 clamped samples in all eight signals). Drills (G 3.9): `DENOISE_DELAY_SAMPLES = 0` made `impulses_stay_aligned` fail ("an impulse moved more than 48 samples: [480, …]"); skipping the last partial block in `chain.rs` made `prop_length_equals_input` fail with the minimal input `n = 1, seed = 0`; one changed character of `manifest_sha256` in `STAMP` made all 7 fixture-reading tests panic "run `pnpm fixtures`" (`--no-fail-fast`), the synthetic cases still passing. All undone; the `duration_preserved.proptest-regressions` file the second drill left was deleted; `pnpm fixtures` says up to date. `cargo clippy -p offcut-dsp --all-targets -- -D warnings`: clean. `cargo build -p offcut-dsp --target wasm32-unknown-unknown --release`: finished. Files: `chain.rs` 175 lines, `denoise.rs` 144, the test files 179 to 277. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `cargo test --workspace` **313 passed** (282 + 31), 0 failed, **3 ignored** (V2's 2 and `chain_timing`), Vitest 92, `pnpm build` exit 0 (`offcut_core_bg` 459,313 bytes, 32 more: `offcut-dsp`'s new modules are linked though nothing calls them yet; `offcut_render_bg` unchanged), `pnpm e2e` 12 passed; the frozen check and the tracked-private check print nothing. CI on this commit is the first run of "Generate the fixtures" on Linux, read before Prompt 68.

**Found, not fixed.** None besides the §19.4 row above.

---

## 2026-10-17 - Prompt 68 (V3 08/30): prosody and native TE-13

**The gate did not stop the build: TE-13 native median 0.707 s per 60 s (`d1`), under 2 s.** CI on Prompt 67's commit `ba0d276`: `ci` and `e2e-media` success; on the runner "Generate the fixtures" took 58 s and `cargo test` 69 s (open item 10: CI's cost is about a minute, D1's 2 to 2.5).

**Done.**

- **`crates/offcut-dsp/src/prosody.rs`** (§7.9, D-96, D-108): `measure_prosody(out48, words, sentences) -> Prosody`. Per word: samples `[start_ms x 48, end_ms x 48)` clamped to the buffer, an empty range giving `ENERGY_FLOOR_DB` and no pitch; energy `10 log10(mean(x²))` floored; pitch on an 8 kHz copy (the mean of each group of 6, `as_chunks::<PROSODY_DECIMATE>`), 320-sample frames at hop 80 wholly inside the word, each with its mean removed and skipped under `VOICED_FRAME_MIN_DB`, normalized autocorrelation over lags 20 to 134, parabolic refinement (the shift clamped to ±0.5), `F0 = 8,000 / τ`; the word's pitch the median of its voiced frames, the lower middle for an even count. Per sentence: population z-scores of energy and of `log2(F0)` over the words with a pitch; fewer than `PROSODY_MIN_WORDS` values or a deviation under 1e-6 give 0.0; a word without pitch gets `pitch_z = 0.0`; `per_word.len() == words.len()`. All sums in `f64`, in a fixed order; no `HashMap`; no indexing (`get`, `zip`). The samples per millisecond and the 8 kHz rate are computed from `SAMPLE_RATE` and `PROSODY_DECIMATE`, not written again.
- **`lib.rs`**: `pub mod prosody`, `pub use prosody::measure_prosody`.
- Inline (§7.10): zero words; a third word 12 dB louder has the largest `energy_z`, above 1.0; a 200 Hz word among 120 Hz words has the largest `pitch_z`, above 1.0; a two-word sentence is 0/0; same inputs, same `Prosody`.
- **`tests/prosody.rs`** (§19.7), the fixture helper of Prompt 67 copied in: the 10 cases of §19.7's table. The F0 case reads 80, 150 and 300 Hz with one and two semitones up; the speech case prints and asserts nothing; the budget case asserts under 2 s only in release.
- **The "brutal" reading** (in `docs/v3/experiments.md`, "Derived clips' events"): on the portrait PCM, "brutal" energy_z +0.91, pitch_z +1.26; "literally" pitch_z +1.79. The README gives sentence times only, so the test spreads each sentence's words over its time by letters: an approximation, read again at S8 with the recognizer's word times.
- **TE-13 native** (G 3.8, in `experiments.md`): `chain_timing` in release, five runs: 0.643, 0.655, 0.707, 0.722, 0.775 s; median 0.707 s per 60 s, `d1`; decision continue, denoise stays on. `measure_prosody` on 60 s with 150 words: 0.148 s in release.
- **Milestone 3**: `pnpm fixtures` up to date; `cargo test -p offcut-dsp` 70 passed, 0 failed, 1 ignored; clippy clean; `wasm32` release build finished; `grep -niE 'silence|pause|gap'` over `src`: comments saying what the crate does not do, `ABSOLUTE_SILENCE`, test names and a test helper's comment; `DENOISE_DELAY_SAMPLES = 480` with "measured", no "(assumption". The four drills of G 3.9: rows 1-3 in Prompt 67, row 4 in Prompt 66.

**Differs from the prompt, the guide or the plan.**

- **The period is the first strong peak, not the plain maximum.** §7.9 says "the best τ* with r ≥ VOICING_MIN". A periodic voice correlates almost as well at two, three and four periods (at 300 Hz the lags 27, 53, 80, 107 and 133 all lie in 20-134), so the plain maximum would halve or quarter the pitch at random. τ* is the first local peak whose `r` is at least `FIRST_PEAK_SHARE` (0.9) of the best and at least `VOICING_MIN`; the frame is voiced only when the best reaches `VOICING_MIN`, as §7.9 says. `FIRST_PEAK_SHARE` is a `pub(crate)` constant of `config.rs` (not one of the 32 public ones). To copy back into §7.9 at Prompt 89 (open item 12).
- `tests/prosody.rs` has 10 cases, the rows of §19.7's table and the prompt's count; G 3.7 says 11.
- `the_budget_for_60_s_and_150_words` carries `#[allow(clippy::disallowed_methods)]` for its clock, as `chain_timing` does.

**Checked.** `cargo test -p offcut-dsp --test prosody -- --nocapture`: 10 passed, the reading printed. `cargo test -p offcut-dsp --lib`: 36 passed. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `cargo test --workspace` **328 passed** (313 + 15), 0 failed, 3 ignored, Vitest 92, `pnpm build` exit 0 (WASM sizes as Prompt 67), `pnpm e2e` 12 passed; the frozen check and the tracked-private check print nothing.

**Found, not fixed.** None besides open item 12.

---

## 2026-10-17 - Prompt 69 (V3 09/30): audio worker and the voice chain in the browser (TE-13)

**The gate did not apply the fallback: TE-13 in the browser, median 733 ms per 60 s in dev and 828 ms in the production build (`d1`), against at most 4,000 ms.** Denoise stays on. CI on Prompt 68's `bbedf47`: `ci` and `e2e-media` success.

**Done.**

- **`crates/offcut-wasm-core/src/audio_api.rs`** (§10.1): `run_chain(pcm48: &[f32], on_block: &Function)` calls `offcut_dsp::run_chain` with `ChainConfig::default()` (D-110). The closure calls `on_block(done)`; a truthy answer is `Flow::Cancel`; a throw sets a flag and cancels, and the call then fails `{ code: "E_INTERNAL", detail: "Callback" }`. `Ok(Some(out))` is the plain object `{ out48, loudness, peak }` with `out48` a new `Float32Array` copied from the samples (its own buffer, so the worker can transfer it), built with `Reflect::set`; `Ok(None)` is `{ cancelled: true }`; `DspError::Empty` / `NonFinite` are `{ code: "E_DSP", detail: "Empty" | "NonFinite" }`. `measure_prosody(out48, words, sentences)` decodes `Word[]` and `Sentence[]` (`E_INTERNAL` `Words` / `Sentences` on failure) and returns the `Prosody` through the crate's `to_js` (`json_compatible`, D-31). No branch on a domain value.
- **`lib.rs`**: `mod audio_api;` only.
- **`web/src/wasm/load-core.ts`** (§13.2): `ChainResult` exported; `CoreApi` gains `runChain(pcm48, onBlock)` and `measureProsody(out48, words, sentences)`, thin wrappers that type the module's object once (`as ChainResult`, `as Prosody`), as V2's wrappers do; no `as <Brand>` of a primitive, so no `MINTING` entry.
- **`web/src/workers/audio.worker.ts`** (§14.1, Appendix C), 49 lines: `kept`; `runChain` sets `kept = null` first (D-90), loads the core, runs the chain with `ctx.progress({ stage: "audio_chain", done, total })` and `ctx.isCancelled()` once per block (a closure, V2 CL item 13), returns `CANCELLED` on a cancel, keeps `out48` and returns `{ loudness }`; `measureProsody` throws `WorkerFailure("E_INTERNAL", "NoAudio")` with nothing kept, measures, clears `kept`, and returns `{ out48, prosody }`, which `serveWorker` transfers. `serveWorker(handlers, { stage: "audio_chain", oneWay: [], duringPreview: [] })`. Imports: `gen/domain` (types), `wasm/load-core`, `workers/protocol` (types), `workers/rpc`.
- **`web/src/workers/pool.ts`** (§13.4): `import audioWorkerUrl from "./audio.worker.ts?worker&url"`, the `WORKERS` row `audio` (stage `audio_chain`), `audio: lazyClient<AudioWorkerApi>(WORKERS.audio)` and its type; `preload()` adds a `modulepreload` link per row, four now. The header comment says four workers.
- **Browser checks** (G 4.3, 4.5), with a temporary `web/tests-e2e/zz-te13.spec.ts`, `web/zz-pw.config.ts` and `web/src/zz-spike.ts` (imported from `main.tsx` for the production build), all deleted afterwards; `main.tsx` was restored from its copy and shows no diff. Readings in `docs/v3/experiments.md` ("TE-13 browser"): first `measureProsody` `E_INTERNAL/NoAudio`; dev 866, 705, 733 ms; production 855, 738, 828 ms with `crossOriginIsolated` true; every run `same: true`, `detached: true`, loudness -14.30 LUFS.
- **G 4.4, bundle size** (`experiments.md`, "Bundle size"): `offcut_core_bg` 459,281 → 698,122 bytes (+238,841); `offcut_render_bg` unchanged; `audio.worker` 15,146 bytes. `check-hosts.mjs`: ok, no new literal, so no `NON_NETWORK_LITERALS` entry and no change to the script.

**Differs from the prompt, the guide or the plan.**

- **`n` is 2,879,992, not 2,880,000.** The portrait PCM is what ffmpeg's AAC decoder gives for the 60 s clip, 8 samples short of 60 s. The check that matters, `out48.length === pcm48.length`, holds.
- **The dev server ran on port 5174.** Port 5173 was held by another process on D1 (PID 43428, which answers every path with an HTML page: not this app's Vite). It was left running. The temporary dev server on 5174 was stopped after the check.
- The page reaches the pool in dev with a dynamic `import("/src/workers/pool.ts")` and in the production build through the spike; a `new Function` would have been refused by the production CSP.

**Checked.** `pnpm build:wasm`; `cargo clippy -p offcut-wasm-core --target wasm32-unknown-unknown -- -D warnings` clean; `pnpm check` exit 0; `pnpm build` exit 0 with `check-hosts` ok. `pnpm e2e:media --grep @no-gpu` on an unkeyed build: "10 passed" (V2's 8 cases and the 2 of `media-setup`), the pool holding its fourth worker. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `cargo test --workspace` 328 passed, 0 failed, 3 ignored (no Rust test is added: TS §5 has no test file for the binding), Vitest 92, `pnpm build` exit 0 (`offcut_core_bg` 698,122 bytes), `pnpm e2e` 12 passed; the frozen check and the tracked-private check print nothing.

**Found, not fixed.** None.

---

## 2026-10-17 - Prompt 70 (V3 10/30): edit-list shapes E0-E4 and probe hardening H1-H8

CI on Prompt 69's `5577dad`: `ci` and `e2e-media` success. Port 5173 on D1 (Prompt 69) was another project's front end, the founder confirmed.

**Done.**

- **State verified before starting**: clean tree on `v3-build` at `5577dad`; `sample_table.rs` 387 lines above its tests (V2 CL known issue 10, as the prompt says); `validate.rs` and `media_api.rs` unchanged since `8a271a9`; fixtures up to date.
- **`crates/offcut-mp4/src/edit_list.rs`** (crate-private, §6.1, D-92), 143 lines above its tests: `EditPlan { edit_offset, presented }` and `resolve_edits(edits, media_timescale, movie_timescale, media_duration)`. E0 (no edits) returns at once, before any timescale check, as V2 did; then at most `MAX_EDITS = 16` entries; one trailing empty edit is set aside (E4); one leading empty edit gives the lead (E2, E3); what remains must be one or more media edits with `media_time >= 0`, else `Unsupported("elst")`. One media edit: `edit_offset = media_time - lead` (saturating, as V2) and `presented = shown(edit)`, V2's E1 and E2 exactly. Two or more (E3): each must start within `CONTIGUITY_TICKS = 1` media tick of where the previous one ended (`abs_diff`, which cannot overflow), the last may run to the end of the media (`segment_duration` 0), and the lengths are summed with `checked_add` (`Malformed("elst")`). `to_media` (`Malformed("mvhd")` for a movie timescale of 0) and `shown` moved here unchanged as methods of a small private `Ticks`. The 12 inline cases of §6.1.
- **`sample_table.rs`**: `edits()` keeps its signature: V2's overflow-checked sum of the `stts` runs, `resolve_edits`, `(plan.edit_offset, plan.presented)`. `Edit` stays with its two fields. 362 lines above its tests (from 387). V2's two edit-list tests pass unchanged; "two media edits are unsupported" stays true because its edits (`media_time` 0 for 1,500 media ticks, then `media_time` 2,000) leave a gap of 500 ticks.
- **`lib.rs`**: `mod edit_list;` (private).
- **`probe.rs`** (§6.3), 210 lines above its tests: **H1** each of the matrix's `a b c d` is taken to the nearest of -1, 0, 1 (16.16) when within 1/256 (256 units) of it; an entry farther from all three gives `R0`. **H3** `max_fps` comes from a private `max_fps(stts, timescale)` that leaves out the last sample's duration when it is a run of its own after other runs (inside a longer run it equals its neighbours, so nothing else changes); `is_vfr`'s spread is unchanged. **H4** was already V2's behaviour (frequency index 15 reads the 24-bit rate from the `AudioSpecificConfig`); it now has its test. H2 and H5-H8: no change, covered by Prompt 71's fixture tests.

**Differs from the prompt or the plan.**

- **H3 is tested through the helper, not through a file.** V2's test MP4 builder (`demux::fixture`, in the frozen `demux.rs`) writes one `stts` run per track, so it cannot make a file whose last sample has a run of its own; `max_fps` was taken out of `video_info` into a private function and tested with the runs directly (89 frames then 1 tick: 30,000; a short frame first: 60,000; one sample: 60,000), plus one case through a file.
- §6.3 says V2 read an explicit rate "from the sample entry"; V2's `audio_specific_config` already read the 24-bit rate. No code change for H4.
- §6.2 describes V2's second edit-list test as two edits at `media_time` 0 and 0; they are at 0 and 2,000. The conclusion (not contiguous, `Unsupported`) is the same.

**Checked.** `cargo test -p offcut-mp4`: lib 39 passed (V2's 24, 12 `edit_list`, 3 `probe`), `mux_roundtrip` 12 passed unchanged. Drill (G 5.5 row 2): `MAX_EDITS = 2` made `e3_three_contiguous_media_edits` and `e3_one_tick_of_mismatch_is_accepted` fail; undone. `cargo clippy -p offcut-mp4 --all-targets -- -D warnings` clean; `pnpm build:wasm`: core 699,226 bytes, render 6,034,084 (both link `offcut-mp4`; about 1 kB each); `git diff --name-only 8a271a9 -- validate.rs demux.rs boxes.rs media_api.rs` empty; `check-file-tree.mjs` ok (TS §5 has listed `edit_list.rs` since Prompt 61). SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `cargo test --workspace` **343 passed** (328 + 15), 0 failed, 3 ignored, Vitest 92, `pnpm build` exit 0 (`offcut_core_bg` 699,226, `offcut_render_bg` 6,034,084 bytes), `pnpm e2e` 12 passed; the frozen check and the tracked-private check print nothing.

**Found, not fixed.** None.

---

## 2026-10-17 - Prompt 71 (V3 11/30): demux and rejection fixture tests

Prompts 70 and 71 were run together at the founder's request, each with its own commit and push. CI on Prompt 70's `f005e9e`: `ci` and `e2e-media` success.

**Done.**

- **`crates/offcut-mp4/tests/demux_fixtures.rs`** (§19.2), 12 cases, with the stamp-checking helper of §6.4 and a `#[serde(deny_unknown_fields)]` mirror of the manifest entry (fields the file does not read are `_`-named with `serde(rename)`, so no `#[allow(dead_code)]` is needed). The accept case reads every `expect.accept` of the manifest (seven `ok_*`, five derived) and asserts each field it holds against the `ClipInfo` and the probed `avg_fps` (duration within 1 ms, the fps range inclusive). The others as §19.2: `ok_vfr.mp4` variable, in range, presentation times strictly increasing; `ok_rotated_90.mov` the recorded rotation, 720x1280, portrait; `ok_60fps.mp4` 60,000 ± 100; `ok_moov_at_end.mp4` with `moov` after `mdat`; `ok_portrait_3x4.mp4`; the two-track fixture's 640x360 track; `speech_60s_portrait.mp4` 60,000 ± 1 ms, 1,800 frames, `mp4a.40.2` 48 kHz stereo; `speech_20s_long_pause.mp4` 20,000 ± 1 ms; every video sample of every `ok_*` read whole (its `stsz` size) with the keyframe flag of `stss` (the `stbl` of the first enabled video track, as the demuxer chooses it); `keyframe_at_or_before` on `ok_60fps.mp4` for 0, 499, 500, 2,999 ms: 0, 0, 30, 150.
- **The metadata walk (D-100)**: every box of each of the five derived clips, from the top, recursively through the containers (`moov trak mdia minf stbl udta edts dinf ilst`, and `meta` both as an MP4 full box and as a QuickTime box), `mdat` skipped (compressed bytes say nothing and could match by chance). No box is named `©xyz` or `©day`, and no other box's bytes hold `©xyz`, `©day`, `com.apple.quicktime.location.ISO6709` or `creationdate`. Passed for all five.
- **`crates/offcut-mp4/tests/probe_rejections.rs`** (§19.3), 15 cases: the eleven single refusals, `rej_h264_444.mp4` with `decode_supported` true (`Ok`) and false (`DecodeUnsupported`), `ok_90s_exact.mp4` (`Ok`, 90,000 ms), and the coverage case. `rejection_of` reproduces `media_api.rs`'s mapping (`NotIsoBmff`/`Fragmented` → `REJECT_CONTAINER`; `Truncated`/`Malformed`/`Unsupported` → `REJECT_CORRUPT`; an I/O failure is no rejection). The table `CASES` holds each `rej_*` fixture but `rej_silent.mp4` with its `decode_supported` and code; the coverage case fails for a manifest `rej_*` entry with no row, for a row whose code differs from the manifest's `expect.reject`, and for a fixture that does not give it. The manifest's `"REJECT_*"` strings deserialize into `RejectReason` itself.
- **Observed and recorded** (G 5.3): `ok_rotated_90.mov` reads **R270** (ffmpeg's `-display_rotation 90` is counter-clockwise; the matrix is a quarter turn the demuxer reads as R270), display 720x1280, portrait: written into the manifest as `"rotation": "r270"`, then `pnpm fixtures` (the stamp changed) and the test again. **`ok_two_video_first_disabled.mp4`'s video track is 640x360**: ffmpeg's `-disposition:v:0 0` clears the first track's `tkhd` enabled flag, so the entry stays and H2 is tested on a real file. Read on the way: `ok_vfr.mp4` 9,967 ms at 22,574 fps x 1,000.
- **`crates/offcut-mp4/Cargo.toml`**: dev-dependencies `serde` (`derive`) and `serde_json`, both workspace dependencies, for the manifest. `Cargo.lock` adds the two to `offcut-mp4`'s entry and changes no version.

**Differs from the prompt or the plan.**

- `crates/offcut-mp4/Cargo.toml` changes, though the prompt lists only the manifest and the changelog: reading `fixtures/manifest.json` "with a small `serde_json` dev struct" needs `serde_json` and `serde`'s derive, which §18.1 gave `offcut-detect` but not `offcut-mp4`. The file is not frozen (§4 marks it changed for `sha2`).
- The manifest gained one value, not two: `ok_vfr.mp4`'s duration and rate were observed in Prompt 63; the two-track entry is kept.

**Checked.** `cargo test -p offcut-mp4 --test demux_fixtures`: 12 passed; `--test probe_rejections`: 15 passed; `cargo test -p offcut-mp4`: 78 passed, 0 failed. Drills (G 5.5): the `rej_mp3_audio.mp4` row removed from `CASES` made the coverage case fail with "rej_mp3_audio.mp4 has no case in probe_rejections.rs"; `fixtures/.gen` moved aside made all 12 `demux_fixtures` cases panic "the generated fixtures are stale or missing: run `pnpm fixtures`" (the folder was moved into the scratchpad and back, never anywhere public); both undone, `pnpm fixtures` up to date. The third row of G 5.5 (`MAX_EDITS`) was Prompt 70's. **Milestone 5**: fixtures up to date; `offcut-mp4` 0 failed; clippy clean; `edit_list.rs` 143, `sample_table.rs` 362, `probe.rs` 210 lines above their tests; no `proptest-regressions` file. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `cargo test --workspace` **370 passed** (343 + 27), 0 failed, 3 ignored, Vitest 92, `pnpm build` exit 0 (WASM sizes as Prompt 70), `pnpm e2e` 12 passed; the frozen check and the tracked-private check print nothing; `git diff 8a271a9 -- validate.rs media_api.rs` is empty.

**Found, not fixed.** None.

---

## 2026-10-18 - Prompt 72 (V3 12/30): word lists, wider Digits, spelled cardinals

Prompts 72 and 73 were run together at the founder's request, each with its own commit and push. CI on Prompt 71's `4b01080`: `ci` and `e2e-media` success (the Linux runner's fixtures, from its own ffmpeg build, give the same rotation, track and metadata results).

**Done.**

- **State verified**: clean tree at `4b01080`; the five public functions of `offcut-text` as the prompt lists. `numbers.rs` has **297** lines above its tests, not the 262 the prompt states (it matters for Prompt 73's 400-line limit).
- **`crates/offcut-text/src/units_lex.rs`** (§8.4; D-84, D-101, D-102): the eight lists verbatim (`CURRENCY_SYMBOLS` sorted by code point `$ £ € ₹`, `CURRENCY_WORDS`, `PERCENT_WORDS`, `MAGNITUDE_WORDS`, `TIME_WORDS`, `DATA_WORDS`, `COUNT_NOUNS`, `COMPARATIVES`) and `COUNT_UNIT_MIN = 100.0`: 9 `pub const`. `CurrencyUnit`, `TimeUnit`, `DataUnit` with `From` into `offcut_types::Unit`. Three crate-private lookups by `binary_search`: `lookup` (a `(word, value)` list), `contains` (a word list), `currency_symbol`. "pound", "pounds", "tb", "terabytes" absent, each asserted. One test per list: strictly ascending (sorted, no duplicate) and lower-case.
- **`tokenize.rs`** (§8.1): a word is `Digits` when, after at most one leading currency symbol (the last character of its leading punctuation, if `$ € £ ₹`), at most one trailing sign or suffix (`k m b x` at the end of the word, or `%` after it, not both), the rest parses with V2's `parse_decimal`; the token's text keeps what was written, lower-cased. `Token` and `TokenKind` unchanged (the doc comment of `Digits` names the new signs). §8.1's ten rows as one inline case, plus "`10k%`: the `%` is punctuation".
- **`cardinals.rs`** (crate-private, §8.2; D-92): `MAX_CARDINAL_TOKENS = 24`, `cardinal`, `point_fraction`. Groups of 0-99 and of hundreds (with an optional "and" before 1-99); magnitudes strictly descending, each at most once; "and" only before 1-99; a teen hundred takes no magnitude; "a" only before "hundred" or a magnitude, and only first; a group after a magnitude is taken only when what follows it is a smaller magnitude or none ("one million two million" is 1,000,000 and 2 tokens); checked `u64` arithmetic. `point_fraction` parses "0.<digits>" from the digit words, so "point zero five" is the `f64` nearest 0.05.
- **`lib.rs`**: `mod cardinals;` and `pub mod units_lex;`.

**Differs from the prompt or the plan.**

- **`#[cfg_attr(not(test), expect(dead_code, reason = …))]`** on `mod cardinals;` and on `units_lex::{lookup, contains}`: nothing outside the tests calls them until `parse_quantity` does in Prompt 73, and an unused private item fails `clippy -D warnings`. `expect` (not `allow`) fails the build once the items are used, so Prompt 73 must remove it, and does.
- **The three unit enums are `pub`, not private** (§8.4 says "private enums of this file"): the `pub const` lists hold their values, and a private type in a public constant is the `private_interfaces` warning. They are not re-exported from `lib.rs`.
- **§8.2's row "nine hundred ninety nine trillion" counts 6 tokens; the words are 5.** Both forms are asserted: without "and" (5, 999 x 10^12), with "and" (6, 999 x 10^12).
- **The 24-token cap fits the largest cardinal without "and"** (exactly 24 words, 999,999,999,999,999); with every "and" it is 29 words, and the cap ends it after the thousands (999,999,999,999,000). Both asserted.
- §8.1's row for `₹2,00,000` prints one `Word`; the tokens are V2's `Punct("₹")` and `Word("2,00,000")` (the symbol is not a letter). What the row means, no `Digits`, holds and is asserted.

**Checked.** `grep -cE '^pub const [A-Z_]+' units_lex.rs`: 9. `cargo test -p offcut-text --lib`: 24 passed (V2's 12 unchanged, 8 list tests, the §8.1 table, 3 cardinal tests holding §8.2's 13 rows and more). `cargo clippy -p offcut-text --all-targets -- -D warnings` clean. Drill (G 6.7 row 1): `"calls"` before `"apps"` in `COUNT_NOUNS` made `count_nouns_are_ordered` fail ("not sorted or duplicated"); undone. `check-file-tree.mjs` ok; `pnpm build:wasm` ok. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `cargo test --workspace` **382 passed** (370 + 12), 0 failed, 3 ignored, Vitest 92, `pnpm build` exit 0 (`offcut_core_bg` 699,976, `offcut_render_bg` 6,034,829 bytes), `pnpm e2e` 12 passed; the frozen check and the tracked-private check print nothing.

**Found, not fixed.** The contradiction Prompt 73 meets: see its entry.

---

## 2026-10-18 - Prompt 73 (V3 13/30): full number grammar, display rules, sentence split

CI on Prompt 72's `c69ba3b`: `ci` and `e2e-media` success.

**The plan miscounts the V2 tests V3 changes.** §8.6 says the "ten million" test is "the only V2 test of the crate whose expectation changes", the prompt's acceptance criterion counts one removed assertion, and §9.9 says the same of one detector test. In fact eleven V2 assertions of `offcut-text` and one V2 test of `offcut-detect` assert V2's stand-ins, which §8.3 and §8.5 replace: each asserted the absence of a V3 feature, the reason §8.6 itself gives for rewriting "ten million". Each was set to the value §8.3/§8.5 give, and no other V2 assertion was touched. The founder asked for Prompts 72 and 73 "carefully end to end"; this was decided in the run and is listed here in full:

| File | V2 assertion | V3, by |
|---|---|---|
| `numbers.rs` | `read(["ten","million"])` = `(1, 10, "10")` | `(2, 10,000,000, "10 million")`: §8.6, the named one |
| `numbers.rs` | `read(["three","point","five"])` = `None` | `(3, 3.5, "3.5")`: §8.3 "point" decimals |
| `numbers.rs` | `read(["2k"])` = `None` | `(1, 2,000, "2,000")`: §8.1, §8.3 suffix |
| `numbers.rs` | `read(["ten","thousand","people"])` = `(2, 10,000, None, "10,000")` | `(3, 10,000, Count people, "10,000 people")`: §8.4 count nouns, D-101 |
| `numbers.rs` | display of 1,234,567 = `"1,234,567"` | `"1.23 million"`: §8.5 |
| `numbers.rs` | display of `3 ,000 ,000` = `"3,000,000"` | `"3 million"`: §8.5 |
| `numbers.rs` | display of `1,250 ,500` = `"1,250,500"` | `"1.25 million"`: §8.5 |
| `numbers.rs` | `format_quantity(1,000,000, Usd)` = `"$1,000,000"` | `"$1m"`: §8.5 |
| `numbers.rs` | `format_quantity(NaN, None)` = `"0"` | `""`: §8.5, §8.7 |
| `numbers.rs` | `format_quantity(800, Milliseconds)` = `"800"` | `"800 ms"`: §8.5 |
| `normalize.rs` | "two hundred and fifty days, 40% less." numbers = `[(2, 6, "250"), (7, 8, "40%")]` | `[(2, 7, "250 days"), (7, 8, "40%")]`: §8.4 time words |
| `offcut-detect/src/lib.rs` | "about ten thousand **people**" gives no event (the test is named "has no unit and is no event") | "people" is now a count noun (a unit, D-101), so the input's last word became "of them"; the assertion, no event, is V2's unchanged. The detector itself is not touched (Prompt 75 rewrites it) |

`git diff 8a271a9 -- crates/offcut-text/src | grep -cE '^-\s*assert'` is therefore 7, not 1 (the other four rewritten values were continuation lines of multi-line `assert_eq!`). One V2 assertion that §8.5 could be read to change was kept: `format_quantity(-0.001)` stays `"0"`, not `"-0"` (a value that shows as zero is not negative).

**Done.**

- **`numbers.rs`** (§8.3, §8.5), 385 lines above its tests (from V2's 297, its spelled-cardinal code moved to `cardinals.rs` in Prompt 72 and removed here). `parse_quantity` in four stages: (1) the core, a `Digits` token (a currency symbol; `%` → `Percent`, `x` → `Times`, `k m b` → ×10^3/10^6/10^9 with no unit; a currency with `%` or `x` is `None`; V2's split groups, D-42, only after a plain core) or a spelled cardinal (`cardinals::cardinal`, "an" read as "a") with an optional `point_fraction`; (2) one magnitude word when the core has none of its own; (3) "grand", ×1,000 and USD unless a currency is set; (4) one unit when the core's sign gave none: "percent", "pct", "per cent", a currency word, "times" before a comparative (consuming "times" only, D-102), a time word, a data word, a count noun (`Count { noun }` as spoken); a currency word after a symbol of the same currency is consumed. `format_quantity` by §8.5's table: currencies through one `money` helper (`$950`, `$10k`, `$1,500`, `$2m`); `None` grouped under a million, else "2 million" / "1.5 billion" / "3 trillion" (a value that rounds to 1,000 of a scale is written in the next: 999,999,999 is "1 billion"); `%`, `x`, ` ms`, singular time and byte words for exactly 1, ` KB`/` MB`/` GB`, the count noun; a negative value `-` and the form of its size; not finite `""`.
- **`sentences.rs`** (§8.6): V2's rule (`by_stops_and_pauses`), then `split_long` per sentence: while a piece has more than 40 words, it is cut after the word followed by its longest gap, the earliest of equal gaps, with a work stack (no recursion). Every word stays in exactly one sentence, in order. §23 item 28 (the tie-break's one-word sentences) is open; the plan's rule is built.
- **`normalize.rs`** (§8.6, D-104): the transcript is tokenized once (token word indices stay global) and the numbers are read per sentence, from each sentence's run of tokens. A word that begins with a comma and three digits with no number before it in its sentence (`,000` after a boundary) is skipped, never read as a number: D-42 says such a word is the next group of a split number and nothing else, and §8.7 says "`,000` alone is no quantity".
- **`lib.rs`, `units_lex.rs`**: the `expect(dead_code)` attributes of Prompt 72 are removed: `numbers.rs` reads the cardinals and the lists.
- Inline tests added: §8.7's `-5` and NaN, a sample of the four stages and of the display table (`$1.5m`, `€500`, `3x`, `$3x` refused, "2 million", "1.5 billion", "three point five million", "five grand", "a thousand people", "40 percent", "12 per cent", "fifty thousand rupees", "$500 dollars", "ten times faster", "three times" with no unit, "ten pounds" with no unit, "1 hour", "90 minutes", "250 gigabytes", "512 bytes", "10,000 subscribers", the unread forms; displays "1 billion", "$1.25m", "€3t", "1.5 days", "1 byte"); §8.7's three split cases; the `$12 / ,000` boundary case.

**Differs from the prompt or the plan.** The table above. Also: §8.7's split case at 650 ms is asserted with its second sentence's start (12,550 ms).

**Checked.** `cargo test -p offcut-text --lib`: 28 passed. `cargo test --workspace`: 386 passed, 0 failed, 3 ignored. Drill (G 6.7 row 3): `normalize_transcript` scanning the whole transcript made `a_number_never_crosses_a_sentence` fail with `[(2, 4, "$12k")]`; undone. `numbers.rs` 385 lines above its tests (it reached 419 on the way: the formatter's currency branch was folded into the main match and the `Digits` core into `Core`, no rule moved out). `cargo clippy -p offcut-text --all-targets -- -D warnings` clean; `pnpm build:wasm`: core 713,091 bytes, render 6,039,008. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `cargo test --workspace` **386 passed** (382 + 4), 0 failed, 3 ignored, Vitest 92, `pnpm build` exit 0 (`offcut_core_bg` 713,091, `offcut_render_bg` 6,039,008 bytes); the frozen check and the tracked-private check print nothing. `pnpm e2e` failed once in the gate with the asset host slow from D1 (the demo video took 23 s; known issue 3) and passed 12 of 12 on the next run, nothing changed between.

**Found, not fixed.** None besides the plan's miscount, which is for Prompt 89's copy-back (open item 13).

---

## 2026-10-18 - Prompt 74 (V3 14/30): number table, properties and sentence tests

CI on Prompt 73's `a84c00d`: `ci` and `e2e-media` success.

**Done.**

- **`crates/offcut-text/tests/numbers_table.rs`** (§19.8): the 62 parse rows and the 10 display rows of §19.8, copied exactly, one tuple per line (`grep -cE '^\s*\("'` gives 72), in two tables, one test each; a failure lists every failing row with what was found and what the table says. Words are split on whitespace, made into `Word`s and tokenized by `tokenize`; the unit is named in the row (`none`, `usd`, …, `count <noun>`). **All 72 rows passed on the first run: no grammar fix was needed.** The two tables carry `#[rustfmt::skip]`, because `rustfmt` wraps the six longest rows onto several lines and G 6.6 counts one row per line.
- **`tests/numbers_prop.rs`** (§19.9), five properties, 256 cases each: digits with separators from 0 to 10^12 read back to the same value, no unit; `format_quantity(n, Usd)` for 1 to 10^9 reads back exactly when the display is exact (under 1,000, whole thousands to 999,000, whole millions) and within 0.5% otherwise; 0 to 999,999,999 spelled by a test-only speller (without "and", so even 999,999,999 is 14 words) is read whole by `parse_quantity`, every word consumed (the public surface: `cardinal` is private); no panic on any sequence of up to 30 words drawn from 40 number words, digits, symbols and fillers, with `1 <= n <= len` and a finite value at every start position; every `Some` displays exactly as `format_quantity` of its value and unit. A sixth test checks the speller.
- **`tests/sentences.rs`** (§19.10): the 8 cases.
- **Milestone 6**: `offcut-text` 44 passed (28 unit, 2 table, 6 property, 8 sentence), 0 failed; clippy clean; no `proptest-regressions` file.

**Differs from the prompt or the plan.**

- The display rows are written unit first (`("usd", 1_000_000.0, "$1m")`), so that they too begin with `("` and the prompt's count of 72 holds.
- §19.9 lists "no panic" and "display is the formatter's" as two properties; they are two `proptest` tests drawing from the same 40-word vocabulary.

**Checked.** `grep -cE '^\s*\("' numbers_table.rs`: 72. `cargo test -p offcut-text`: 44 passed, 0 failed. Drill (G 6.7 row 2): `("pounds", CurrencyUnit::Gbp)` added to `CURRENCY_WORDS` in sorted place made the table fail on one row, `"ten pounds": found Some((2, 10.0, Gbp, "£10")), the table says Some((1, 10.0, None, "10"))`; undone from a copy. Clippy found hand-written `%` multiple checks in three lines of the property test (`is_multiple_of` instead); fixed. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `cargo test --workspace` **402 passed** (386 + 16), 0 failed, 3 ignored, Vitest 92, `pnpm build` exit 0 (`offcut_core_bg` 713,091, `offcut_render_bg` 6,039,008 bytes, unchanged: tests only), `pnpm e2e` 12 of 12; the frozen check and the tracked-private check print nothing. A first gate run, started before the clippy fix, stopped at `pnpm check` (exit 101) and was run again whole.

**Found, not fixed.** None. `offcut-text` is complete for V3 (§21.2).

---

## 2026-10-18 - Prompt 75 (V3 15/30): detector configuration, lexicons and NumberReveal V3

CI on Prompt 74's `79daf63`: `ci` success (its `e2e-media` job included).

**Labels first (G 7.1).** Labels frozen at `9c96667` on 2026-10-17: `git log --format=%ct -1 -- fixtures/labeled` gives `1792189206`, the commit of Prompt 65; `git ls-files crates/offcut-detect/src | grep -E 'from_to|list|keyword|exclusions|resolve'` printed nothing before this prompt's first edit; `git status --short fixtures/labeled` prints nothing before and after. No file of `fixtures/labeled/` was opened while the rules were written (D-87).

**Done.**

- **`crates/offcut-detect/src/config.rs`** (§9.1, D-84): V2's ten fields with their names, comments and values unchanged; the 29 V3 fields appended in §9.1's order with its table's defaults; `threshold(kind)` and `hold_ms(kind)` (1,400 / 1,700 / 1,700 / 0, a KeywordPop holding nothing); the 11 lexicons as sorted `const` slices, every word compared with §9.1's table by a script (11 of 11 equal, in order); `ORDINAL_MARKERS` is `&[(&str, u8)]` sorted by word. No rounding constant: the founder chose none (§23 item 27, Prompt 62). Inline: the defaults (V2's ten and every V3 value), the four thresholds and holds by kind, every lexicon sorted, unique and lower-case with its size, every ordinal within `list_count_max`, and no list noun a count noun of `offcut-text` (D-101).
- **`src/number.rs`** (§9.3, D-106, D-107, D-109): `Candidate.parts`; `Found { words, tokens, quantity, sentence, suffixed }`; `quantities` reads each sentence's effective tokens with `parse_quantity` the way `normalize_transcript` reads the transcribed words (one quantity's tokens never start another; a headless `,000` is skipped), so that without edits it gives exactly `t.numbers` (an inline test holds that). `suffixed`: a `k`, `m` or `b` suffix, a magnitude word of `offcut-text`, or "grand". `find` keeps V2's signature and scores 0.50 + unit 0.30 (any unit but `None`, and `Count` only from `COUNT_UNIT_MIN` = 100) + magnitude 0.20 (at least 1,000, or `suffixed`) + energy 0.10, clamped, times the least word confidence; `params` carries the D-107 label. Inline: the edit `3000` → `4000` (one `Found` of 4,000 hours at the same words and tokens), an edit making a number where there was none, hidden words, the five Script A rows of §9.3 (quantity, unit, display and label each), the label's stops, the score terms, and `parts`.
- **`src/lib.rs`**: the V2 test §9.9 names (`ten_thousand_dollars_at_confidence_0_9_is_one_number_reveal`, which holds the edited and hidden case) is the one V2 test rewritten. Two of its assertions change: the edit to `$20,000` now gives one NumberReveal of 20,000, `$20k`; and its label is `there`, not `None` ("we made $10,000 there.": D-107 takes the word before the full stop). The hidden half still gives none, and the edit elsewhere still changes nothing. `detect` itself is V2's, now with the V3 `number::find`, until Prompt 77.

**Differs from the prompt or the plan.**

- **The edited number keeps its id** (open item 14). §9.9, §19.11's edit row and §19.13 say the re-read number gets "a new id", and TS §17.6 that its toggle state is not inherited. An id is FNV-1a over the kind and the anchors only (D-47, whose nine bytes "never change"), and an edit of one word keeps the anchors, so the id is the old event's. The rewritten test asserts what the code does (same anchors, same id) and names the open item.
- **A hidden word removes the number it was part of, not only the quantity it is inside.** D-109: "a hidden word inside it yields none". Read only as "inside the quantity read again", hiding `,000` of "$12 ,000" would leave a NumberReveal of `$12` over a voice saying twelve thousand. `quantities` drops a quantity whose own words hold a hidden word, and also one that overlaps a transcribed number with a hidden word (the founder's rule: a missed animation is fine, a wrong one is not). Both cases are inline tests.
- **A label stops at a number in digits too.** D-107 names punctuation, a stop verb and the sentence end, and takes "Word tokens"; a `Digits` token after the quantity ends the label rather than being stepped over ("90 minutes on 3 drafts" → `on`).
- **The two V2 scoring constants moved into `config.rs`** as `NUMBER_MAGNITUDE_MIN` (1,000) and `NUMBER_EMPHASIS_Z` (1.0), from `number.rs` (§9 "Never": no threshold outside `config.rs`). They are `const`s, not fields: the 39 fields are frozen from here (§21.1). `number.rs` keeps two words of grammar (the suffixes `k m b` and "grand"), which say what `suffixed` means rather than tune anything.
- **The multi-line lexicons carry `#[rustfmt::skip]`**, several words to a line. `rustfmt` put one word per line (`STOP_WORDS` alone took 104 lines), and `config.rs` reached 413 lines above its tests, over the limit of 400 of `check-file-tree.mjs`; now 234. The lexicons stay in `config.rs`, as D-84 and the prompt's count require.

**Checked.** `grep -cE '^\s+pub [a-z_]+:' config.rs`: 39. `grep -cE '^pub const [A-Z_]+: &\[' config.rs`: 11. `cargo test -p offcut-detect --lib`: 20 passed (4 of `config.rs`, 7 of `number.rs`, V2's 7 of `lib.rs` and 2 of `event_id.rs`). Before the rewrite, V2's tests run against the new `number::find` failed in exactly one test, the one §9.9 names, at its label assertion. `cargo clippy -p offcut-detect --all-targets -- -D warnings` clean. `git status --short fixtures/labeled`: nothing. Drill (G 7.2): `("first", 1)` swapped before `("fifth", 5)` in `ORDINAL_MARKERS` made `every_lexicon_is_sorted_unique_and_lower_case` fail ("ORDINAL_MARKERS is not sorted or has a duplicate"); undone, the file byte for byte its copy. One score row compares within 1e-6: 0.50 + 0.30 + 0.10 is 0.90000004 in `f32`. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0 (`check-file-tree.mjs` ok), `cargo test --workspace` **413 passed** (402 + 11), 0 failed, 3 ignored, Vitest 92, `pnpm build` exit 0 (`offcut_core_bg` 713,091 bytes, unchanged; `offcut_render_bg` 6,052,943, 13,935 more: it carries the detector), `pnpm e2e` 12 of 12; the frozen check and the tracked-private check print nothing.

**Found, not fixed.**

- Open item 14 (the id of an edited number), for the founder before Prompt 77.
- Open item 15: `CLOCK_WORDS` holds `a.m.` and `p.m.` as §9.1 lists them, but `tokenize` makes `a.m.` the Word `a.m` and the Punct `.`, so those two entries never equal a token. For Prompt 77's X3.

---

## 2026-10-18 - Prompt 76 (V3 16/30): ListReveal, FromTo and KeywordPop finders

CI on Prompt 75's `8bae8e4`: `ci` and `e2e-media` success. Draft pull request #5 mergeable, no conflict.

**The founder's two pending decisions, delegated to the agent on 2026-10-18** ("take these decisions yourself, prioritizing customer experience"). Both are written into the plan in this commit, so that Prompts 77, 78 and 80 read them.

- **Open item 14, an edited number keeps its id: accepted.** A toggle lives in `EditState.event_overrides`, keyed by `EventId` (TS §8, D-82), so an id that survives the edit keeps the user's on/off choice. For the customer: a number switched off stays off when its misheard text is corrected, instead of coming back on by surprise; one left on shows the corrected value; any later per-event choice keyed by the id survives a typo fix too. The alternative (resetting `enabled` when `params` change) would undo a choice the user made. Changed: v3implementation §9.9's paragraph, §19.11's edit row, §19.13's edit row, §23.2 (TS §17.6's edit bullet joins the copy-back list); Prompt 77's `rules_table.rs` line and Prompt 78's `redetect.rs` line of `coding-promptsv3.md`. The open item stays for the TS copy-back at Prompt 89.
- **§23 item 26 (open item 3), records outliving their clip: `clear()`.** Each repo gains `clear()`, and import step 4 calls the three beside the D-40 sweep, so the device holds one clip's records at most, as OPFS holds one clip's media. For the customer: a clip's words and voice measurements do not outlive the clip, and until V7 nothing reads a record of an earlier clip, so nothing is lost. §23 item 26 now reads "Decided"; Prompt 80 applies it and updates §13.5 and §15.1 (its own instructions for this choice). Entry 1's decisions row updated.

**Done.**

- **`crates/offcut-detect/src/list.rs`** (§9.5; D-105, D-107), 379 lines above its tests. `pub(crate) struct Marker { ordinal, words, ordinal_word, at, sentence }`; `pub(crate) fn markers`; `pub(crate) fn sentence_tokens` (each sentence's token range, also used by `keyword.rs`). A marker is read at a sentence's first token after any `FILLERS` and punctuation ("So, second, …"): an `ORDINAL_MARKERS` word; "number" and a cardinal or digits; or a cardinal or digits directly followed by `,` `.` `:`. The places a cardinal may name are 1 to the highest ordinal of `ORDINAL_MARKERS` (7), read with `parse_quantity` on that one token (whole, no unit), so no number word is listed outside `offcut-text`. `find`: the announcer form (score 0.60 + 0.30, + 0.10 when every marker is an ordinal word; header `"{count} {NOUN}"`), then the marker-only form (0.65, + 0.20 when no gap between markers' starts exceeds 15 s; header empty). Markers must name 1, 2, 3, … one after the other, each before the list's start + 45 s. Items: up to 5 words after the marker, leading fillers dropped, stopping at `. ? ! ; :`, at `,` once two words are taken, or at the sentence end. `anchors` from the count (or first marker) to the last item's last word; `parts` the count and each marker.
- **`src/from_to.rs`** (§9.4; D-85, D-106), 172 lines. Per sentence, each later quantity within 8 Word tokens, a different value, compatible units after inheritance (same variant; time group; data group; a count only with its noun; two currencies never). Frame: "from" right before `a` and "to" right before `b` (+0.25), else a `CHANGE_VERBS` word among the 3 tokens before `a` in its sentence or between, with "to" right before `b` (+0.15). +0.15 same unit, +0.05 inherits, +0.05 gap at most 6, clamp, x the least confidence of both quantities' words. Best `b` per `a`, the nearest of equal scores. The side that inherited is written again with `format_quantity` ("40" becomes "40%"), the one call outside `offcut-text` (D-106).
- **`src/keyword.rs`** (§9.6), 86 lines. One candidate per Word token of at least 4 letters, no stop word, not a word of a quantity or of a marker; 0.50 + 0.20 x clamp(energy_z - 1) + 0.20 x clamp(pitch_z - 1) + 0.20 for an intensifier right before it in its sentence or an emphasis word; clamp, x the word's confidence; `params = KeywordPop { word }`, `parts` empty.
- **`src/config.rs`**: three `pub const` counts the rules name, `FROM_TO_VERB_REACH` (3), `LIST_MARKER_ONLY_MIN` (3), `LIST_ITEM_COMMA_WORDS` (2), and the grammar words `FROM`, `TO`, `NUMBER`, so that no threshold or word of the finders lives outside `config.rs`. None is a field (the 39 are frozen) and none is a lexicon slice (the count of 11 holds). The punctuation that marks a marker or ends an item stays in `list.rs` as two private lists.
- **`src/lib.rs`**: `mod from_to; mod keyword; mod list;`, each with `#[cfg_attr(not(test), expect(dead_code, …))]`: `detect` still calls only `number::find`, and an unused private item fails `clippy -D warnings`. `expect` fails the build once the finders are used, so Prompt 77 removes the three attributes (the pattern of Prompt 72).
- **Inline tests, 19 new.** `list.rs` 9: Script A (1.00, `3 REASONS`, the three items, anchors, parts, item times); no third, and a third 45.8 s after "three" (none) against one at 45.7 s (one); marker-only 0.85, 0.65 with a 22 s gap, none for two markers or for digit markers alone; "It was the first time I tried." (no marker) and §9.9's "First, … It was the first time …" (one marker); digit and "number" markers; "I have 3 tips. 1, sleep. …" (0.90, `3 TIPS`); "eight reasons" (none); items' fillers and commas; an announced list that leaves a later marker alone. `from_to.rs` 6: the seven rows of §9.4 at their scores (2019-2024 at 1.00 and "from 3 to 5 people" at 0.90 before exclusions, `3 people`), §19.11's "Costs dropped 40% to 25%." (0.90), unit groups, gaps of 7, 8 and 9 words, best partner, anchors, parts and confidence. `keyword.rs` 4: the five rows of §9.6, the excluded words, the intensifier's place, anchors and confidence.

**Differs from the prompt, the guide or the plan.**

- **"Same explicit unit" is the same `Unit` variant from both quantities' own words, `None` and `None` included.** §9.4's check column says "explicit", but its table gives "from 2019 to 2024" 1.00 (0.55 + 0.25 + 0.15 + 0.05), D-85's wording ("when both carry the identical variant from their own words") gives the same, and the prompt asserts 1.00. Read the other way, a bare pair with a from frame would score 0.85, exactly the threshold, and pass or fail on `f32` rounding (item 27's problem). Open item 16.
- **A list announcer of any count claims the markers that follow it.** §9.5's table says "eight reasons" with eight ordinal markers gives none "because 8 is over `list_count_max`", but no ordinal word names an eighth place, so the marker-only form would find "first" to "seventh" and show a list of seven under a speaker who announced eight: the partial list TS §17.6 calls worse than none. Every count-and-list-noun phrase therefore claims the markers 1, 2, … after it (up to its count when in range); only an in-range count with all its markers is a candidate, and the marker-only form skips claimed markers. The same rule keeps "five tips" with four markers from becoming a four-item list. Open item 16.
- **Item words include numbers in digits.** D-107 says "Word tokens"; an item "we saved $10,000" would otherwise read "we saved", which says less than the speaker did. Open item 16.
- **A list's score is multiplied by the least confidence of its anchor words.** §9.5 does not say; TS §17.4 ("multiplied by the minimum ASR confidence of the anchor words") does, for every kind. Since TE-2 every confidence is 1.0.
- **Strict order.** The markers of a list are the ones that follow one another (1, then 2, …), not any later marker of the right number: a stray marker in between ends the run. A missed list over a wrong one.
- Punctuation and fillers before a marker are both skipped ("So, first, …"), the reading of "the first effective Word token of a sentence, after any `FILLERS`".

**Checked.** `cargo test -p offcut-detect --lib`: 39 passed (20 + 19), first run. `cargo clippy -p offcut-detect --all-targets -- -D warnings` clean, and for `wasm32-unknown-unknown`. Lines above the tests: `config.rs` 250, `event_id.rs` 30, `from_to.rs` 172, `keyword.rs` 86, `lib.rs` 79, `list.rs` 379, `number.rs` 235. Drill: the lead of a sentence widened to skip every token up to an ordinal word (mid-sentence markers) made `a_marker_is_only_the_first_word_of_a_sentence` fail on "It was the first time I tried." (and the digit-marker case); undone from a copy, `cmp` equal. No file of `fixtures/labeled/` was opened; `git status --short fixtures/labeled` prints nothing. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `pnpm test`: Rust **432 passed** (413 + 19), 3 ignored, Vitest 92; `pnpm build` exit 0 (`offcut_core_bg` 713,091, `offcut_render_bg` 6,052,943 bytes, both unchanged: the finders are not called yet); `pnpm e2e` 12 of 12; the frozen check and the tracked-private check print nothing.

**Found, not fixed.** None besides open item 16.

---

## 2026-10-18 - Prompt 77 (V3 17/30): exclusions, resolution, full detect, rules table

CI on Prompt 76's `5b01293`: `ci` and `e2e-media` success.

**Done.**

- **`crates/offcut-detect/src/exclusions.rs`** (§9.7), 87 lines. `apply(cands, toks, t, edits)` zeroes a candidate when one of its parts (matched to its `Found` by word range, so the unit is the quantity's own, before a FromTo's inheritance) is: X1 a bare whole number 1900-2099 in one plain `Digits` token; X2 a `VERSION_CONTEXT` word right before, a `VERSION_AFTER` word right after, or two decimal points; X3 a `CLOCK_WORDS` word right after; X4 a bare plain run of 7 digits or more; X5 bare and under 10 in absolute value (NumberReveal and FromTo parts); X6 a NumberReveal part equal to a marker's words; X7 a hidden word inside the anchors (every kind; an edit that is empty or only spaces, which `tokenize` also turns into no token). A ListReveal's parts are checked by X7 only, so its announcer count is exempt from X5. The only file that reads `edits` itself.
- **`src/resolve.rs`** (§9.8, Appendix C), 259 lines. Step 0 stable sort: priority FromTo, ListReveal, NumberReveal, KeywordPop; score descending (`total_cmp`); `span.start`; `anchors.start`; `anchors.end`. Steps 1-2 in one walk: a candidate is dropped when its anchors intersect a fixed event's or a kept candidate's, or, for an overlay, when its window `[start - 150, end + hold(kind)]` overlaps or touches one. Step 3: while some `overlay_window_ms` interval from a window start holds more than three window starts, the lowest score among its kept candidates goes (the later of equal scores); fixed events count and stay. Step 4: one KeywordPop per sentence (the higher score, then the earlier), then in time order a pop under 2,500 ms after the previous one goes unless its score is higher, in which case the previous goes; a fixed pop stays and beats a close candidate. Returns the kept candidates only; never changes a score or an anchor.
- **`src/lib.rs`** (§9.2): `detect` in seven steps: `tokenize` with edits; the four finders in the order from_to, list, number, keyword (each reads the quantities again, D-104, D-109); `exclusions::apply`; the threshold by kind (after `settled`, below); `resolve(cands, &[], cfg)`; events with `event_id(kind, anchors)`, `enabled: true`, sorted by `span.start`, then D-47's kind order, then `anchors.start`. The three `expect(dead_code)` attributes of Prompt 76 are gone; `exclusions` and `resolve` are private modules like the finders. V2's seven tests of `lib.rs` pass unchanged (the FromTo candidates of "$5,000 or $7,000" score 0.75 and fall at the threshold, so V2's two window cases still keep the earlier amount).
- **`src/config.rs`**: `CLOCK_WORDS` holds `a.m` and `p.m` (open item 15; still five words, sorted); named constants `YEAR_FIRST`, `YEAR_LAST`, `VERSION_DOTS`, `PHONE_DIGITS_MIN`, `BARE_NUMBER_MIN`, `SCORE_NOISE`. No field changed; the 11 lexicon slices stay 11.
- **`Candidate.sentence`** (`number.rs`, set by the four finders): the sentence of the candidate's first word, for step 4.
- **`tests/rules_table.rs`** (§19.11), the synthesis helper of §17.5 (word length `clamp(120 + 45 x letters, 150, 700)`, 80 ms within a sentence, 400 ms after `. ? !`, first word at 500 ms, `*word*` emphasized at 2.0/2.0, confidence 1.0) with a per-sentence earliest start for the rows that need a pause. One table test runs 32 rows and prints `32 rows of section 19.11 run here, 0 failed`; row 31 (INV-6) is asserted for every event of every row; row 32 (the edit) is its own test and asserts `$20k` with **the same id** (open item 14, decided 2026-10-18); three more tests check rows 1-2's labels, row 13's items and row 28's spacing (2,065 ms). Row 21, "*Workflow* matters.", asserts **0.90**: item 27 is decided, so the score is not on the threshold.
- **Inline tests**: `exclusions.rs` 7 (X1-X7, including §9.9's hidden list-item word: X7 zeroes the list, a hidden word outside it does not); `resolve.rs` 5 (§9.9's fixed NumberReveal at 10 s drops a new one at 10.5 s; priority; touching windows; the 10 s cap with a fixed event; pops per sentence and 2.5 s apart). §9.9's `markers` row passed since Prompt 76.

**Differs from the prompt, the guide or the plan.**

- **`lib.rs` step 5 removes `f32` noise before the threshold.** `0.65 + 0.20` is `0.84999996` in `f32` (measured on D1), under `threshold_list` 0.85, so no marker-only list could ever pass: row 16, TS §17.4's marker-only form and three frozen labels (`w06`, `w15`, `w29`) would all be dead. `settled` takes a score within `SCORE_NOISE` (1e-6) of a whole hundredth as that hundredth, in this one place, with its constant in `config.rs` (the prompt's acceptance box for item 27's rounding). This is not item 27's rejected rounding: a score really between two hundredths (0.795) is not moved, so the founder's "a missed animation is fine, a wrong one is not" still holds; only the sums of the config's two-decimal values are read as the values they stand for. Every event's confidence is the settled score, so INV-6 holds. Open item 17.
- **`Candidate` gains `sentence`** (`number.rs`, `from_to.rs`, `list.rs`, `keyword.rs`, though the prompt lists `lib.rs` only): `resolve`'s fixed signature receives no transcript, and step 4 needs each pop's sentence. A fixed pop is never in a candidate's sentence (`redetect_sentence` fixes only events wholly outside its sentences), so fixed pops count for the 2.5 s gap only. `Candidate` is crate-internal (D-106's reasoning). Open item 17.
- **`CLOCK_WORDS` changes** (`config.rs`), open item 15's second option.
- **Drill 2 as worded does not fail row 10.** Letting touching windows pass leaves row 10's windows overlapping by 1,180 ms, so both are still not kept; the touching rule is pinned by V2's `of_two_amounts_600_ms_apart_only_the_earlier_one_is_kept` and `resolve`'s `windows_that_touch_clash…`, which both failed. Letting overlapping windows pass made row 10 fail as G 7.6 says (`$2k` and `$20k` both kept).
- Row 29's displays are `$1k`, `$2k`, `$3k` (§8.5's money form); my first draft of the row said `$1,000` and failed; the plan's row states only "at most three".
- Row 30 uses an announced list ("Here are three lessons. First, we saved $10,000. …") rather than a marker-only one, so that it does not depend on step 5's noise rule.

**Checked.** `cargo test -p offcut-detect --lib --test rules_table -- --nocapture`: lib 51 passed (39 + 12), `rules_table` 5 passed with the line above. Drills (G 7.6), each from a copy and restored (`cmp` equal): X1 off → row 11 failed (`FromTo "2,019 -> 2,024"`) and two inline tests; touching windows allowed → two inline tests (above); overlapping windows allowed → row 10 failed and three inline tests; `threshold_list = 0.60` → row 17 failed (`ListReveal "3 []" 0.65`) and two config tests. `cargo clippy -p offcut-detect --all-targets -- -D warnings` clean, and for `wasm32-unknown-unknown`. Lines above the tests: `config.rs` 267, `event_id.rs` 30, `exclusions.rs` 87, `from_to.rs` 173, `keyword.rs` 91, `lib.rs` 95, `list.rs` 381, `number.rs` 239, `resolve.rs` 259. `git status --short fixtures/labeled` prints nothing; no labeled file was opened. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `pnpm test`: Rust **449 passed** (432 + 17), 3 ignored, Vitest 92; `pnpm build` exit 0 (`offcut_core_bg` 713,091 bytes, unchanged; `offcut_render_bg` 6,106,226, 53,283 more: `detect` now calls the three finders, exclusions and resolution); `pnpm e2e` 12 of 12; the frozen check and the tracked-private check print nothing. **Milestone 7**: every row green, drills fired and undone, no label changed.

**Found, not fixed.** None besides open item 17.

---

## 2026-10-18 - Prompt 78 (V3 18/30): `redetect_sentence` and determinism

CI on Prompt 77's `ecf6b96`: `ci` and `e2e-media` success. Item 17's `f32` noise fix (`settled`) confirmed by the founder on 2026-10-18.

**Done.**

- **`crates/offcut-detect/src/lib.rs`** (§9.2, D-86), 185 lines above the tests (95 before). `detect` is split into `candidates` (steps 1-5) and `into_events` (step 7, beside fixed events, sorted as before); its output is unchanged. `pub fn redetect_sentence(t, p, edits, s, prev, cfg)`, the frozen signature: `s` out of range returns `prev` unchanged; else the scope (below), `outside` = the events of `prev` that share no word with it, steps 1-5 over the whole transcript, the candidates wholly inside the scope, `resolve(cands, &outside, cfg)`, and `outside` plus the kept candidates as events in `detect`'s order. Ids come from `event_id(kind, anchors)` as in `detect`, so an event whose kind and anchors did not change keeps its id (D-47).
- **The scope** (private `scope`): sentence `s`; every sentence whose `start_ms` lies in `(start_ms(s), start_ms(s) + list_window_ms)` (45 s of time, D-86); then the sentences of every event of `prev` that shares a word with the scope, repeated until none crosses its edge. It is always whole sentences, so a fixed KeywordPop is never in a candidate's sentence (`resolve`'s step 4 relies on this).
- **`tests/redetect.rs`** (§19.13), 7 cases on Script A as recognized: the 17 sentences of `a01-scriptA.json` with §17.5's synthesized timing, 157 words, `detect` giving the `3,000 hours` NumberReveal, the FromTo, the `3 REASONS` list and `$12k` (no KeywordPop: no prosody). Sentence numbers count from 0; the README's sentences 3, 8 and 9-11 are 2, 7 and 8-10 here. (1) no edit, every sentence 0-16: equal to `prev`. (2) word 23 to `4000`, sentence 2: `4,000 hours` with the old id, the other three unchanged, and equal to `detect` of the edited text. (3) word 79 hidden, sentence 8: the list is gone, nothing else changes. (4) word 75 to `four`, sentence 7: no list (three markers); back to `three` with that `prev` (no event touches sentence 7): only the 45 s bring sentences 8-10 into the scope, and the list returns with its old id and `3 REASONS`. (5) the fixed event wins (below). (6) sentences 17, 18 and `u32::MAX`, and an empty transcript: `prev` returned as it is. (7) the time of one call, printed.
- **`tests/determinism.rs`** (§19.14), 3 cases: (1) the 31 labeled files with data (written ones synthesized, `edits` applied; `a02` skipped), `detect` twice equal, ids and order included; (2) proptest, 256 cases: up to 200 words from a vocabulary of number words, list nouns, ordinals and markers, fillers, stop words and frames, emphasis words, digits and amounts with their units, and the X1-X4 traps (G 8.2), plus 18 phrases (three whole lists, announcers, markers, three FromTo frames, traps); random word lengths, pauses up to 30 s, confidences 1.0/0.95/0.85/0.6, prosody in ±3, up to three random edits or hidden words. Each case checks two calls equal, anchors inside `0..words.len()`, `span.start <= span.end`, sorted output, INV-6, no shared words, no two overlay windows meeting, at most one KeywordPop per sentence, and `redetect_sentence` of a random sentence with nothing changed giving `prev` again; (3) every labeled file built twice from the same raw words: equal transcripts, equal events.
- **The timing helper** of `rules_table.rs` is copied into both files, as the prompt says (`determinism.rs`'s copy returns the raw words so that case 3 can build twice).

**Differs from the prompt, the guide or the plan.**

- **The scope closes over the events of `prev` that cross its edge** (open item 18). D-86 and §9.2 step 1 take the sentences of the events that touch `s` only. An event of `prev` that reaches into the scope without lying wholly inside it (a list announced in the last sentence that starts within 45 s of `s`, its markers after) is then not `outside`, so not kept, and not wholly inside, so its candidate is dropped: the event is lost on an edit somewhere else. The labeled files are all under 45 s and never show it; the proptest does. With the plan's rule taken literally, case 2 failed within 256 runs (`redetect_sentence` with no change returned less than `prev`); with the closure it passes, also at 2,000 runs. Signature and every other step are the plan's.
- **§19.13's fixed-event row is played on the FromTo** (open item 18). The plan puts a new amount in sentence 1 beside `3,000 hours` (sentence 2). A sentence starting within 45 s after `s` is in the scope, so that event is detected again, not fixed; and the first sentence after it starts 35 ms too late (`3,000 hours` is shown until 12,050 ms; `This` at 12,235 ms opens its window at 12,085 ms). The case edits word 60 (`That`, first of sentence 6) to `$5,000`: its window opens inside the window of the FromTo of sentence 5 (outside the scope, fixed), and the result equals `prev`. With the FromTo taken out of `prev`, the same call keeps the amount, so it was the fixed event that took its place.
- **The proptest vocabulary has phrases.** With single words only, 1,000 generated transcripts gave 3,410 NumberReveals, 2,291 KeywordPops, 1 FromTo and 0 lists: the overlay and scope checks never met a list. With the phrases: 1,527 / 1,648 / 1,786 FromTo / 1,018 ListReveal, 892 of the 1,000 over 60 s.
- **`#[allow(clippy::disallowed_methods)]` on the timing test only**, as `offcut-dsp/tests/prosody.rs` does: `clippy.toml` bans `Instant::now` everywhere, and §19.13 asks for a printed time. The crate itself reads no clock.
- The 7 + 3 cases are the plan's; checks the plan does not list (no shared words; re-detection with nothing changed) are inside the existing cases, not new ones.

**Checked.** `cargo test -p offcut-detect --release --test redetect -- --nocapture`: 7 passed; **142-290 µs per call** over 100 calls on Script A (157 words), five runs on D1, release (budget 50 ms on R1, M1.5, not asserted). `cargo test -p offcut-detect --test determinism`: 3 passed. `cargo clippy -p offcut-detect --all-targets -- -D warnings` clean, and for `wasm32-unknown-unknown`. Drill (G 8.5 row 2): `resolve`'s order cut to `b.score.total_cmp(&a.score)` → `resolve::tests::shared_words_go_to_the_higher_priority_then_the_higher_score` and `windows_that_touch_clash_and_the_earlier_of_equals_stays` failed; `determinism.rs` passed, as it should (a stable sort by score alone is still deterministic, it only chooses other winners); the guide's "or a test of event order fails" is the branch that fired. Undone from a copy; `git diff` shows no change of `resolve.rs`. The literal-scope check above was also run from a copy and undone. No `proptest-regressions` folder or `*.proptest-regressions` file remains (proptest writes `tests/determinism.proptest-regressions` beside the test; the two written by the literal-scope runs were deleted). The labeled files were read by the two tests, as specified; none changed (`git status --short fixtures/labeled` prints nothing). SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `pnpm test`: Rust **459 passed** (449 + 10), 3 ignored, Vitest 92; `pnpm build` exit 0 (`offcut_core_bg` 713,091 bytes, unchanged; `offcut_render_bg` 6,106,820, 594 more: `detect`'s split; `redetect_sentence` is not exported until Prompt 79); `pnpm e2e` 12 of 12; the frozen check and the tracked-private check print nothing.

**Found, not fixed.** None besides open item 18.

---

## 2026-10-18 - Prompt 79 (V3 19/30): precision gate and the redetect binding

CI on Prompt 78's `f8fb1c5`: `ci` and `e2e-media` success. From this prompt on the agent pushes each commit to `v3-build` itself (the founder's instruction of 2026-10-18); `main`, merges and tags stay Human.

**The gate: passed at the default thresholds.** Overall precision **0.93** (40 true positives, 3 false positives), recall 0.98, over **30 files read, 1 skipped** (`a02`, no data yet). Per kind: NumberReveal 0.80 / 1.00, ListReveal 1.00 / 1.00, FromTo 1.00 / 0.92, KeywordPop 1.00 / 1.00 (precision / recall). No threshold changed; nothing for TS §17.4. The table, the sweep and the false positives are in `experiments.md`, `Precision gate`.

**Done.**

- **`crates/offcut-detect/tests/precision_recall.rs`** (§19.12), one test. Reads every `fixtures/labeled/*.json` with `serde_json` into structs with `deny_unknown_fields` (`tags` and `edits` optional; `author`, `frozen` and `notes` read and checked by type only); `schema` 1 and `id` = file name asserted; a file that does not parse fails with its name. An `asr` file with `words: null` is skipped and printed `skipped (no data yet)`. Written files go through §17.5's synthesis (the helper of `rules_table.rs`, copied), `asr` files as they are, both through `normalize_transcript(raw, "labeled")`; `edits` applied. SCHEMA's units mapped to `Unit` (`count:<noun>` to `Unit::Count`; an unknown unit fails with the file name). `say` matched by SCHEMA's normalisation on whole words (a run starts on a word's first piece and ends on a word's last; `occurrence` picks the n-th; an absent `say` fails with the file name). The coverage check of §17.5 runs first, from `expected` (8 per kind, 6 files with none) and `tags` (X1-X5 two each, the announcer and marker-only forms 5 and 3, from-frame and change-verb 4 each, no-change pairs 2, years in a from-to, a partial list, an `edit_number` file with a number edit and a `hidden_word` file with an empty edit); a shortfall names every missing item. Matching is greedy in spoken order per file and kind: the unmatched expectation with the highest IoU, at least 0.5, and equal values and units (NumberReveal, both ends of a FromTo). Printed: the per-kind and overall table, the sweep, the false positives as `file: kind "words" (confidence)`, the misses, and labels or headers that differ (reported, not matched). Three assertions: at least 30 files, INV-6 for every prediction, overall precision at least `DETECTOR_PRECISION_TARGET` (`offcut-types/src/limits.rs`, found by `git grep`), with a message pointing to §20.3. 0.86 s in a debug build.
- **The sweep runs `detect` again** for each step (one kind's threshold moved, the others at their defaults). The default run has already dropped everything under the default thresholds, and a lower threshold changes what `resolve` keeps, so filtering the default output would not show what §20.3's raise would ship.
- **`crates/offcut-wasm-render/src/detect_api.rs`** (§10.2): `redetect_sentence(transcript, prosody, edits, sentence: u32, prev)`, decoding with the existing helpers (`Transcript`, `Prosody`, `Edits`, `Events` as detail, all `E_INTERNAL`), `SentenceIdx::new(sentence)`, `DetectorConfig::default()`; no product rule. `EventId` decodes from the string it is serialized to, so a `prev` that came out of `detect` goes back in.
- **`web/src/wasm/load-render.ts`** (§13.3): `RenderApi.redetectSentence(transcript, prosody, edits, sentence: SentenceIdx, prev: readonly DetectedEvent[])`, a copy of `prev` passed to the module. Nothing calls it; `render.worker.ts`'s `redetectSentence: notYet,` is unchanged (frozen).
- **`docs/v3/experiments.md`**, `Precision gate`: question, method, the printed output, the false positives by file, date, decision, labelled `d1`.

**Differs from the prompt, the guide or the plan.**

- **Drill 1 does not fire as worded.** With `threshold_number = 0.50` the gate prints one new false positive (`w07-camera: number_reveal "4K" (0.70)`), and overall precision is 0.91 (NumberReveal 0.75), still at or above 0.90, so the test passes. The exclusions zero most low-scoring numbers before the threshold, so the number threshold alone lets little through. The same drill on the keyword threshold (`threshold_keyword = 0.50`) fails the gate: 131 new KeywordPop false positives, overall 0.23, the §20.3 message printed. Both undone from a copy (`cmp` equal).
- **Reported beyond §19.12's list**: the misses (false negatives) and labels or headers that differ, so the record shows recall's causes too.

**Checked.** `cargo test -p offcut-detect --test precision_recall -- --nocapture`: 1 passed, the output above. `cargo test -p offcut-detect`: 67 passed, 0 failed. `cargo clippy -p offcut-detect -p offcut-wasm-render --all-targets -- -D warnings` clean, and `offcut-wasm-render` for `wasm32-unknown-unknown`. `pnpm build:wasm` exit 0; `offcut_render.d.ts` exports `redetect_sentence`. `git grep -n "redetectSentence" -- web/src`: `load-render.ts` (type and implementation), `protocol.ts`, and `render.worker.ts`'s `notYet` line. Drill 2 (G 8.5 row 3): `w01`'s `"value": 48000` changed to 49000 without a changelog line; `git status --short fixtures/labeled` showed ` M fixtures/labeled/w01-saas-pricing.json`, and the test still passed (one more false positive and one miss, overall 0.91): only the D-87 review catches it, as the guide says. Undone; `git status --short fixtures/labeled` prints nothing. No label changed. Milestone 8: `pnpm fixtures` up to date; `cargo test -p offcut-detect` 0 failed; precision at least 0.90 over 30 files read; the labeled files unchanged since their commit. SAB gate: no `zz` file, no test key, `pnpm check` exit 0, `pnpm test`: Rust **460 passed** (459 + 1), 3 ignored, Vitest 92; `pnpm build` exit 0 (`offcut_core_bg` 713,091 bytes, unchanged; `offcut_render_bg` 6,113,457, 6,637 more: `redetect_sentence` is exported); `pnpm e2e` 12 of 12; the frozen check and the tracked-private check print nothing.

**Found, not fixed.**

- **"pushed" is not a change verb.** `w28`'s "40 seconds, and fixing those pushed it up to 3 minutes" is labeled as one FromTo; with no change-verb bonus the pair scores under 0.85, and its two ends are shown as two NumberReveals instead (all three false positives of the gate come from that sentence and `w19`'s "about 20 minutes", each at exactly 0.80). Not changed here: the labeled set must not be used to tune a rule (D-87). An input for V4's detector work, with the sweep's reading that NumberReveal at 0.85 has precision 1.00 and recall 0.92.

---

## 2026-10-18 - Prompt 80 (V3 20/30): IndexedDB repos and import-clip records and rejection event

CI on Prompt 79's `8acbc06`: `ci` and `e2e-media` success.

**Done.**

- **`web/src/persistence/clips-repo.ts`** (§13.5, D-81, D-103), 85 lines. `HASH_PREFIX_BYTES = 4_194_304`; `hashPrefix(file)`: WebCrypto SHA-256 over `file.slice(0, 4 MiB)` followed by `file.size` as an unsigned 64-bit little-endian integer (`DataView.setBigUint64(…, true)`), lower-case hex; a file the browser cannot read rejects with the browser's error. `put(record)`, `get(clipId)`, `clear()`; `toBytes(n)`, the file's one cast, the last `return` of a function declaration (the `MINTING` entry of Prompt 62).
- **`web/src/persistence/transcript-repo.ts`** and **`edits-repo.ts`**, 57 lines each: `put(clipId, record)`, `get(clipId)`, `clear()`, with V1's `TranscriptRecord` and `EditsRecord` (no new type).
- **All three**: key `clipId`; value `Versioned<T>` with `schemaVersion: DB_VERSION` (1), as `entitlement-repo.ts` writes it; a stored value with a newer `schemaVersion` is treated as absent and deleted (TS §23.3); every IndexedDB failure, `openDb` included, is rethrown as `OpfsError`: `quota` for a `DOMException` named `QuotaExceededError`, `io` for anything else (D-103). No logging; imports only `gen`, `db.ts`, `opfs.ts` and `schema.ts`. The few lines of the `Versioned` get/put and the error mapping are repeated in each file: the one shared home would be `db.ts`, frozen in V3.
- **§23 item 26, `clear()` (decided 2026-10-18; open item 3 closed).** Each repo's `clear()` removes every record of its store; `import-clip.ts` step 4 calls the three beside the D-40 sweep, a failure ignored as the sweep's is. The device holds the records of one clip at most, as OPFS holds its media. "Start over" alone removes nothing; the next import does. §13.5 (signatures and a `clear()` row) and §15.1 (step 4) of the plan updated in this commit, as the prompt says for this choice; §15.2 step 9's "stays until V7" now says "until the next import clears it".
- **`web/src/usecases/import-clip.ts`** (§15.1), 249 lines: step 6 `sourceHashPrefix = await clipsRepo.hashPrefix(file)` before `importAndProbe` (a `NotReadableError` is `E_STORAGE_IO`, stage `import`, through `importFailed`; any other error is `E_INTERNAL` and rethrown, as step 7's non-worker errors are); step 8 on `rejected` sends `clip_rejected { reject_reason }` with the code alone (D-97, INV-18); the record `{ clipId, createdAt, lastOpenedAt, clipInfo, sourceBytes: toBytes(file.size), sourceHashPrefix }` with `now = Date.now()`, a failed `put` failing the import with `E_STORAGE_QUOTA` or `E_STORAGE_IO`, stage `storage`, `retryable: true`, and removing `clipDir`. The file name is never read. The doc comment points at §15.1 instead of V2 §19.2.

**Differs from the prompt, the guide or the plan.**

- **The record is written before `accepted`, not after `clip_accepted`.** §15.1 put `clipsRepo.put` at step 10, after `clipStore.accepted`, with `clipStore.failed(…)` on failure. The clip machine (`state/machines/clip-machine.ts`, TS §12.2, untouched in V3) lets `accepted` go only to `processing` or `idle`: `failed` from there throws `IllegalTransitionError`. The write is now step 9, while the clip is `importing`, whose `fail` edge exists; `accepted` and `clip_accepted` are step 10, so a clip whose record cannot be written fails the import and is not counted as accepted. §15.1's table says so (updated with the `clear()` change).

**Checked.** `pnpm check` exit 0 (eslint, both `tsc`, `check-file-tree.mjs`: the three files are in TS §5). Drill (G 9.1): `export const b = 5 as Bytes;` appended to `transcript-repo.ts` made eslint fail ("A cast to a unit or id type is allowed in gen/ and workers/ only"); restored from a copy, `cmp` equal. Browser check on the dev server (Chrome, D1), a temporary `web/zz-pw.config.ts` and `web/tests-e2e/zz-import-records.spec.ts`, both deleted (the asset host routed to abort, so no model was fetched): (1) dropping `fixtures/.gen/rej_4k.mp4` sent one batch with `landing_view`, `capability_check` and exactly one `clip_rejected` `{"reject_reason":"REJECT_RESOLUTION"}`, no `clip_accepted`; the `clips` store stayed empty and the rejected stub showed; (2) dropping `ok_vfr.mp4` stored `{"schemaVersion":1,"value":{"clipId":…,"createdAt":…,"lastOpenedAt":…,"clipInfo":{…},"sourceBytes":3360576,"sourceHashPrefix":"d6be3464a5db2d3be28de1083fd71a36fe53c98aef8cf3db9a61b9a6e12d2b1c"}}`: `sourceBytes` the file's size, the hash equal to the D-81 hash computed in Node from the file, exactly the six fields, no `clip.mp4`; after "start over" the record was still there, and the next import (`rej_4k.mp4`) left the store empty (item 26). The dev server's proxy logged one `ECONNREFUSED` for a request that reached it with no local API (the batch a closing page sends; the checked batch went to the in-page fake). `wc -l`: `import-clip.ts` 249, `clips-repo.ts` 85, `transcript-repo.ts` 57, `edits-repo.ts` 57. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `pnpm test`: Rust **460 passed**, 3 ignored, Vitest 92 (unchanged: no test file is added, §13.5); `pnpm build` exit 0 (`offcut_core_bg` 713,091, `offcut_render_bg` 6,113,457 bytes, both unchanged); `pnpm e2e` 12 of 12; the frozen check and the tracked-private check print nothing.

**Found, not fixed.**

- **A clip failed at stage `storage` during import looks to "start over" like one with a render session.** `dismissClip`'s `sessionOpened` (V2) counts `failed` at `detect_scene` or `storage` as having opened a session, so after a failed record write it calls `previewPause` and `closeSession` on the render worker, which starts it for nothing; both calls are caught. Harmless and rare (a full or failing IndexedDB at import); the stage `storage` is the plan's. Left for V4's review of `dismissClip`.

---

## 2026-10-18 - Prompt 81 (V3 21/30): run-pipeline with the voice chain, prosody and records

CI on Prompt 80's `7dbe851`: `ci` and `e2e-media` success.

**Done.**

- **`web/src/usecases/run-pipeline.ts`** (§15.2, C-4, D-90), 349 lines (298 before). Both `V3:` stand-ins are gone.
  - **Step 4 (B2):** right after `extractAudio`, `pushFeed({ kind: "cleaning_voice" })`. Then `pool.audio.runChain({ pcm48 }, { transfer: [pcm48.buffer] })`, its `Cancelled` turned into a failure by `answered`, wrapped by `settle`, which never rejects and gives `{ ok, at } | { failure, at }`. On `ok`, `stage_timing audio_chain` is tracked, measured on the main thread from the call to the settling (D-35).
  - **Step 5:** V2's `out48 = owned(audio.pcm48)` is gone; the audio worker owns `pcm48`.
  - **Step 6:** a model failure waits for the chain, then `failed(modelFailure)`.
  - **Steps 7-8 (B1, C):** `cacheInfo`, `load`, `transcribe`, the `asr` timing and `unload` run in one `try`. Any failure there waits for the chain, unloads the model, and throws the earlier failure by time. A fault of the code is never hidden behind the chain's failure.
  - **Step 9:** no speech waits for the chain, then `noSpeech()`, `clip_rejected { reject_reason: "REJECT_NO_SPEECH" }`, `clipDir` removed.
  - **Step 10:** the chain's failure is thrown, stage `audio_chain`.
  - **Step 11 (D):** `measureProsody({ words, sentences })` on the cleaned voice; `out48` comes back.
  - **Step 12 (E):** detection with the real prosody. A FromTo's feed line has `display` = `from.display` and `displayTo` = `to.display` (new `displayToOf`, D-83); the `assertNever` default stays.
  - **Step 13 (F):** `opfs.writeAudio`, then `transcriptRepo.put(clipId, { transcript, prosody })`, then `editsRepo.put(clipId, { editState: edit })`.
  - **Step 14:** `ready` and `pipeline_done`, as V2.
  - `failureOf`, `answered`, `owned`, the model wait and `runPipeline` are unchanged. The cancel and `client_error` notes of V4 stay.
- **Every path from the chain's start to step 10 waits for it** (D-90): the model failure (6), any throw in the joined B1 block (7-8, including `cacheInfo` and the closing `unload`, which V2 had outside its `try`), no speech (9), and step 10 itself. Every failure path ends in one store transition.
- **`crates/offcut-detect/src/config.rs`, `lib.rs`:** seven comments that began `V3:` (labels of what V3 added, from Prompts 73-77) are reworded ("FromTo, new in V3 (…)", "As of V3, …"), so that `git grep -n "V3: " -- web/src crates` prints nothing, as the prompt checks. No code changed.
- **`web/tests-e2e/model-download.spec.ts`**: **REOPENED CONTRACT (D-91, extended by the founder on 2026-10-18).** Five assertions said the feed's first line is "Transcribing…". The chain starts beside the model wait (D-90), so "Cleaning voice" comes first, and four `@no-gpu` cases failed. The fifth assertion is in the non-`@no-gpu` case. D-91 named only three `pipeline-preview.spec.ts` cases, so the founder was asked. The five now go through one helper, `expectFeedBegun`: the feed's first two lines are "Cleaning voice" and "Transcribing…", in either order (G 9.5's rule). Nothing else in those cases changed. D-91's row in the plan records the extension.

**Differs from the prompt, the guide or the plan.**

- **The B1 `try` holds `cacheInfo` and the closing `unload` too**, so that no throw leaves the chain unawaited (D-90's "no path leaves the chain running"). V2 had both outside; a failure of either is still stored as before, after the join.
- **D-91 extended** (above). Without it `pnpm e2e:media --grep @no-gpu` gives 6 of 10 (the prompt expects V2's 8 cases to pass), and CI's `e2e-media` would fail on every push.
- **Comment wording in `offcut-detect`** (above): the prompt's grep covers `crates`, where V3's own labels used the marker's form.
- `git grep -n "V4: " -- web/src` prints one line (`video-source.ts`), not the prompt's expected list. The render worker's handlers and the use-case's cancel and retry places say "V4" in prose (`notYet`, "the cancel arrives in V4", "before V4"), not as `V4:` markers. Nothing removed.

**Checked.**

- **Browser check (G 9.5), dev server, Chrome on D1.** A temporary `web/zz-pw.config.ts` and `web/tests-e2e/zz-pipeline-v3.spec.ts`, both deleted. The model was put on the context's device from `fixtures/.cache`, the asset host answered locally, the API a fake.
- **Reference clip:** `ready` after 61.8 s.
  - Feed, recorded with a `MutationObserver` from before load: "Cleaning voice", "Transcribing…", "Found: 3,000 hours", "Found: 90 minutes to 10 minutes", "Found: 3-item list", "Found: $12k". No KeywordPop: "brutal" did not pass on the measured prosody, which the plan allows.
  - `stage_timing`: `probe_audio` 708 ms, `audio_chain` 1,283 ms, `asr` 59,588 ms (`webgpu`, dev build), `detect_scene` 115 ms.
  - `pipeline_done`: `total_ms` 60,908, `n_number` 2, `n_list` 1, `n_from_to` 1, `n_keyword` 0. No `model_download`, no `clip_rejected`.
  - IndexedDB: one record each in `clips`, `transcripts` and `edits`, under the same key. `transcripts` has 157 words and 157 `per_word` entries; `edits` holds V2's default `EditState`.
  - OPFS `out48.f32`: 14,343,360 bytes.
  - The 3 request bodies hold 23 strings; none is a word of the transcript, or holds one of six letters or more.
- **`rej_silent.mp4`** after "Start over": the rejected stub; one more `clip_rejected`, `{"reject_reason":"REJECT_NO_SPEECH"}`; `clips/` empty in OPFS. Its `clips` record stays (item 26: the next import clears it), and the reference clip's `transcripts` record is gone (cleared by this import).
- **`pnpm e2e:media --grep @no-gpu`** (unkeyed build), three runs:
  - The first gave 5 passed. Cases 3-6 failed in the model download itself (no model request recorded, no aborted request within 120 s, "could not be downloaded" where the hash message was expected), and case 7 on the feed's first line.
  - Two runs since showed no download failure: the requests case alone passed (13.6 s), and a full run gave 6 passed, its four failures all the feed's first line.
  - The first run started right after a full `pnpm build`; it is recorded as a load-related flake on D1, not reproduced. With D-91 extended: **10 passed** (1.7 min).
- **The rest:** `wc -l web/src/usecases/run-pipeline.ts` 349; `git grep -n "V3: " -- web/src crates` prints nothing. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0 (run again after the spec change), `pnpm test`: Rust **460 passed**, 3 ignored, Vitest 92; `pnpm build` exit 0 (`offcut_core_bg` 713,091, `offcut_render_bg` 6,113,457 bytes, both unchanged); `pnpm e2e` 12 of 12; the frozen check and the tracked-private check print nothing.

**Human (optional):** listen to the preview of the reference clip: the voice should sound cleaner and stay in sync.

**Found, not fixed.**

- **ASR took 59.6 s for the 74.7 s reference clip in the dev build on D1** (about 48 s per 60 s, under D-67's 145 s line, over E-3's 20 s target). The dev server serves unminified modules; E-3 is read on the production build by the bench at S12 (Prompt 88).
- The full `pnpm e2e:media` (not only `@no-gpu`) still has the three D-91 cases of `pipeline-preview.spec.ts` to rewrite (Prompt 85); they assert V2's stand-ins, as the SAB expects between Prompts 73 and 85.

---

## 2026-10-18 - Prompt 82 (V3 22/30): recognized transcripts of the derived clips and the V3 test clip

CI on Prompt 81's `4259c42`: `ci` and `e2e-media` success.

**Done.**

- **One run of each clip through the V3 pipeline on D1.** `pnpm dev`, Chrome, the model on WebGPU, one Playwright worker, a persistent profile in `fixtures/.cache/zz-profile-p82` (git-ignored) so the model was written once. A temporary `web/zz-pw.config.ts` and `web/tests-e2e/zz-recognized.spec.ts` dropped each clip, waited for `ready`, recorded the feed from before the page loaded, read the analytics batches from the in-page fake, and read the newest clip's transcript from IndexedDB by its `clipId` (G 9.6). The outputs went to the session's scratchpad, never the repository. The spec, the config and the profile are deleted. All eleven ran: 11 passed, 10.7 min. Eight clips: the five derived clips, `v3testclip.mp4` and the founder's three new recordings (below). Three checks: `rej_4k.mp4`, `rej_silent.mp4` (Milestone 9).
- **`fixtures/speech/README.md`, "The derived clips as recognized".** The table of five clips (accepted as, feed, §17.3's expectation, as recognized) and each clip's sentences with word ranges and times, numbers and events. The 60 s clips share one table (identical words and times).
- **Two expectations are changed to the clips as recognized, each with its reason; the detector is not changed** (§20.1):
  - **The 60 s clips give no ListReveal.** The recognizer writes the three items as one sentence, so "second" and "third" are not at a sentence start (D-105), and "three reasons" is a partial list.
  - **The long-pause clip's second part starts at 9,960 ms, not 10,000 ms or later**: within the recognizer's 20 ms steps and a word's onset (the room tone runs 5.000-10.000 s).
- **`speech_20s_noevents.mp4`** gives no "Found:" line, and `speech_30s_sparse.mp4` no event, as expected. No KeywordPop on "brutal": +0.56 energy, +0.57 pitch on the cleaned voice.
- **"The V3 test clip as recognized".** The 12 recognized sentences with word ranges and times; where they differ from the founder's words ("Mayan Bhattag"; sentence 2 reworded; "SAS"; "right?" a sentence of its own; `$2 ,000` as two words; "the plan"; sentences 9 and 10 joined); the numbers; the events with their reasons. Its expected feed for Prompt 85: "Cleaning voice" / "Transcribing…" in either order, then "Found: 3 months", "Found: $2k", "Found: 4 months", "Found: that's" (the last depends on prosody). **The clip is accepted and reaches `ready` (88 s): the second recording passes (§20.1).** Nothing about its location tag is written.
- **`docs/v3/experiments.md`.** "Derived clips' events" gains its S8 reading (the per-clip table, "brutal", the two differences). "Second recording" is filled: accepted, `ready`, timings, the events against `a02`'s labels, and the founder's three further recordings (events only).
- **Milestone 9's rows.** `rej_4k.mp4`: rejected in 0.4 s; the events request holds one `clip_rejected` `{"reject_reason":"REJECT_RESOLUTION"}` and no `clip_accepted`. `rej_silent.mp4`: `clip_accepted`, then one `clip_rejected` `{"reject_reason":"REJECT_NO_SPEECH"}`. The reference clip's three records were read in Prompt 81. The README holds the six entries.

**The founder's three new recordings** (2026-10-18, `testclips/`, git-ignored, never uploaded). The files are `v3testclip1 (1).mp4` (script 2, the traps), `v3testclip2 (2).mp4` (script 3, noise and a pause) and `v3testclip3 (3).mp4` (script 1, all four kinds). The scripts suggested `v3script1.mp4` to `v3script3.mp4`; the files were left as named. All three are webcam recordings, 1280x720, variable frame rate, H.264 and AAC, 45-69 s. **Each carries a GPS location tag**, so they stay on D1 (D-100); no coordinate is written anywhere. All three are accepted and `ready`.

- **Script 1:** 10,000 subscribers, 6 hours to 45 minutes, a 4-item list, 3x found. $80 is inside the list's last item, 40% is written `40 %`, and UNREAL and ENOUGH are missed.
- **Script 2:** 400 ms and $240 to $60 found. The marker-only list is run into the sentence before, and WHOLE is missed. **"5 million views" is shown for "1.5 million views".**
- **Script 3:** ₹50k, 6 months to 2 weeks, 120 clients found. 250 GB is written `250GB` and CHEAPEST is missed. **"7 minutes" is shown for "4.7 minutes".**
- **No trap of scripts 1 and 2 became an event.**
- Their transcripts are not committed (D-112 names the founder's two clips); the events are in `experiments.md`.

**Differs from the prompt, the guide or the plan.**

- **The two README expectations changed with their reasons** (above), as the prompt says for an expectation that does not hold.
- **Three more clips than the prompt names were run** (the founder's recordings of 2026-10-18), recorded as further readings of "Second recording".
- §17.3's sparse row ("under 20% speech") is read from how the clip was built (five 1.1 s pieces, 18%), not from the recognizer's times, which stretch the first piece over the room tone.

**Checked.** Milestone 9: `pnpm build:wasm` exit 0, `pnpm check` exit 0, Vitest 92 passed, `cargo test --workspace` 0 failed, `pnpm e2e` 12 passed, `pnpm e2e:media --grep @no-gpu` 10 passed (V2's 8 and the 2 of `media-setup`), `run-pipeline.ts` 349 and `import-clip.ts` 249 lines, the frozen check empty. `git grep -c "as recognized" -- fixtures/speech/README.md`: 6; no "filled at S8" placeholder is left. SAB gate: no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm test`: Rust **460 passed**, 3 ignored, Vitest 92; `pnpm build` exit 0 (`offcut_core_bg` 713,091, `offcut_render_bg` 6,113,457 bytes, unchanged); the tracked-private check prints nothing. No labeled file changed.

**Found, not fixed** (recorded, not tuned to: D-87, §20.1).

- **Open item 19: a decimal written with its point as a separate word is shown with the wrong value** ("4 .7 minutes" as 7 minutes, "1 .5 million" as 5 million). The founder decides V3 or V4. Prompt 88's precision gate will also count `a02`'s three unexpected events.
- **KeywordPop rarely fires on real speech.** None of the six capitalized words of the scripts (UNREAL, ENOUGH, WHOLE, CHEAPEST) nor "brutal" passed, and one unexpected pop did ("that's", pitch_z 1.68). An input for V4's look at the emphasis weights, with E-7.
- **Speech said quickly runs sentences together** (the list items of the derived clips and of script 2, "Fourth … costs $80 …" in script 1), and the list and number rules that read sentence starts lose events there. Recorded for V4.
- ASR on these clips in the dev build: 0.98 to 1.4 s per second of audio. E-3 is read on the production build at Prompt 88.

---

## 2026-10-18 - Outside a prompt: the split decimal point (open item 19), and renders of the test clips

**The founder's decision on open item 19 (2026-10-18): fix it in V3.** The founder's words: "if we catch 80 percent [of the] events, lists and all, we are good"; the confidence of a wrong-or-right call goes up later with LLM-based edit plans that have the clip's context. So a wrong value is fixed now, and missed events (emphasis, a split `%`, a unit run into its number) are left. Prompt 88 applies §20.3 as written if `a02` brings the overall precision under 0.90; the founder accepts that a raised threshold may cost some real events.

**Done.**

- **`crates/offcut-text/src/numbers.rs`.** The recognizer writes a decimal with its point as a new word (`4 .7 minutes`, `1 .5 million views`), as it writes `$12 ,000` (D-42). `with_split_groups`, which joins split thousands groups after a plain whole number, now also takes one word of a point and digits only, straight after the number, as its fraction, and ends the number there. A point that ends a word ("3." then "5") stays a full stop; a fraction with a sign or suffix (`.7%`), a word in between, a core that already has a point or a sign are left as they were. One function, one rule: the two splits share their "a punctuation word right after the number, then digits" test. 400 lines above the tests: the rule went into the loop it shares its conditions with, not a function of its own, to stay within TS §29 (a first version as its own function made the file 413 lines, refused by `check-file-tree.mjs`).
- **One inline test**, `a_number_the_recognizer_split_at_its_decimal_point_is_one_number`, 13 assertions: the founder's two cases (`4.7 minutes`; `1.5 million views`, count noun "views"), `$4.99`, `$1,250.5` (groups, then a point), `5 .5 .5` read as 5.5 (one point), and the six cases that are not joined. V2's D-42 test passes unchanged.

**Checked.**

- **On the eight real transcripts** of Prompt 82, with their measured prosody: a temporary test in `offcut-detect`, deleted after. Exactly two events changed: "5 million views" became **"1.5 million views"** (script 2) and "7 minutes" became **"4.7 minutes"** (script 3). Every other event of the eight was the same, and no trap became an event (`3 .11`, `3 .12` stay versions, X2; `5 .5 .5 .1 …` stays small bare numbers, X5).
- **In the app:** the two recordings run again through `pnpm dev` with the new module, whose feeds read "Found: 1.5 million views" and "Found: 4.7 minutes".
- **The labeled set:** `precision_recall.rs` 0.93 overall, unchanged (its decimals are one word each).
- **SAB gate:** no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `pnpm test`: Rust **461 passed** (460 + 1), 3 ignored, Vitest 92; `pnpm build` exit 0 (`offcut_core_bg` 713,306 bytes, 215 more; `offcut_render_bg` 6,113,666, 209 more: both bundles carry `offcut-text`); `pnpm e2e` 12 of 12; the frozen check and the tracked-private check print nothing.

**Renders for the founder** (local only, `testclips/renders/`, git-ignored). Six clips were processed in `pnpm dev` with a Creator export (a temporary Playwright file and profile, deleted; the test key given to that dev server's process only, never written to a file, D-24). V3 draws NumberReveal only; the other kinds are detected (the feed shows them) and drawn from V4.

- `v3-reference-scriptA.mp4`, `v3-derived-60s-portrait.mp4` and `v3-v3testclip.mp4`.
- `v3-v3testclip1-script2.mp4`, `v3-v3testclip2-script3.mp4` and `v3-v3testclip3-script1.mp4`, before the fix.
- `v3-v3testclip1-script2-fixed.mp4` and `v3-v3testclip2-script3-fixed.mp4`, after it.
- Each export took 16-38 s after `ready`.
- The founder's recordings carry location tags and stay on D1 (D-100).

**Found, not fixed.**

- Known issue 4, the preview attaching while an export locks it (above), seen while making the renders.
- Open item 20: the copy-back of the split point into the plan and TS.

---

## 2026-10-19 - Prompt 83 (V3 23/30): rejection panel and the 14 rejection messages

CI on `ab48dba` (the split decimal point): `ci` and `e2e-media` success.

**Done.**

- **`web/src/copy/messages.ts`** (§16.3, D-80), 338 lines. `rejections`: the 14 entries of §16.3's table word for word, each `{ title, body, action }`, `satisfies Record<RejectReason, ErrorCopy>`.
  - Every limit is computed from `LIMITS`, none typed into a sentence (INV-15): `MAX_FILE_SIZE / BYTES_PER_MB` (500 MB), `MAX_CLIP_DURATION / MS_PER_SECOND` (90 seconds), `MAX_LONG_SIDE` (1920 pixels), and `MAX_INPUT_FPS / FPS_MILLI_PER_FPS` (60 frames per second, in both the body and the action of `REJECT_FRAME_RATE`, through one `MAX_FPS`).
  - `BYTES_PER_MB = 1_000_000` and `FPS_MILLI_PER_FPS = 1000` sit beside V2's `MS_PER_SECOND`.
  - `editor.rejectedStub` is removed; `failedStub` and `startOver` stay.
  - None of the six P-12 phrases occur.
- **`web/src/ui/components/RejectionPanel.tsx`** (new, §16.2), 34 lines: `<section role="alert" data-testid="clip-rejected" data-reject={code}>`, an `h2` with the title, two paragraphs (body, action), and a "start over" button calling `dismissClip()`. The copy is read by key from the store's `reject`, with no `switch`. No retry, no code, size or file name shown. With no `reject` in the store (a state the store never makes), only the button shows.
- **`web/src/ui/pages/EditorPage.tsx`**: the `rejected` arm of `body()` is `<RejectionPanel />`. V2's `Stub` still serves `failed`.
- **`web/src/ui/styles/components.module.css`**: `.rejection` composes V2's `.stub` from `pages.module.css` (the first `composes:` of the repository; the built panel carries both classes, a flex column with a 16 px gap). `.rejectionTitle` has `.problem`'s size and colour, `.rejectionBody`, and `.rejectionAction` in the muted colour. No V2 class changed; no `style` attribute.
- **`scripts/check-copy-codes.mjs`**: the 14 `REJECT_*` entries and their comment are removed from `COPY_PENDING`. The check prints `REJECT_REASONS 14 of 14`; pending falls from 38 to 24 (V2's `E_*` codes including `E_DSP`, and the four blockers).

**Differs from the prompt, the guide or the plan.**

- **`web/tests-e2e/landing.spec.ts` changed: its one reader of `rejectedStub`.** §16.1 removes the key "with its last reader", and the prompt lists no test file, but V2's case "a dropped file starts an import and opens /app, or is refused where Offcut cannot run" read `messages.editor.rejectedStub`, so `tsc -p tests-e2e` would fail. Its assertion "the page says so, and offers the way back" now reads the panel: visible, a `data-reject` code that has copy, its heading equal to that code's title, and the "start over" button. Which code the case's 15 bytes of text get (`REJECT_CONTAINER` or `REJECT_CORRUPT`) is the reader's, so the case does not pin one. Nothing else in the case changed; it passes in `pnpm e2e`.
- **`grep -c "REJECT_" scripts/check-copy-codes.mjs` prints 1, not 0**: the one line is `{ name: "REJECT_REASONS", file: DOMAIN }` in `LISTS`, which tells the script where the codes are. No `REJECT_*` code is pending, which is what the check means.

**Checked.**

- **The prompt's commands:** `grep -oE '\bREJECT_[A-Z_]+:' web/src/copy/messages.ts | sort -u | wc -l` 14; the forbidden-phrase grep prints nothing; `git grep -n "rejectedStub" -- web` prints nothing; `node scripts/check-copy-codes.mjs` ok.
- **Drills (G 10.4)**, each from a copy and restored (`cmp` equal):
  - `REJECT_AUDIO_CODEC` deleted: `tsc` fails (TS1360, the `satisfies` of `rejections`, and TS2551 in `RejectionPanel.tsx`), and `check-copy-codes` fails with "REJECT_AUDIO_CODEC: has no copy in messages.ts and is not in COPY_PENDING".
  - "90 seconds" typed into `REJECT_DURATION`'s body: `grep -n "90 seconds"` finds line 314.
  - A sentence in `RejectionPanel.tsx`'s JSX: ESLint fails, `react/jsx-no-literals`.
- **Browser check (G 10.3)**, `pnpm dev` in Chrome on D1, a temporary `web/zz-pw.config.ts` and `web/tests-e2e/zz-rejections.spec.ts`, deleted:
  - `rej_4k.mp4`: "This video is larger than Offcut handles", with 1920.
  - `rej_hevc.mov`: "This video uses HEVC", with "Most Compatible".
  - `rej_too_long_91s.mp4`: "This clip is too long", with 90 seconds.
  - `rej_120fps.mp4`: "This video has too many frames per second", with 60.
  - Each panel had `role="alert"`, its `data-reject` code, and its heading, body and action. The page text held neither "clip.mp4" (the name `dropClip` gives every file), nor the fixture's name, nor the code.
  - "Start over" brought the drop zone back each time; `ok_vfr.mp4` after them was accepted (the feed showed).
  - A screenshot of the HEVC panel showed the title in the problem colour, the two paragraphs and the button.
- **Milestone 10:** `pnpm check` exit 0, the copy check, `pnpm build`, `pnpm e2e` 12 passed, `pnpm e2e:media --grep @no-gpu` 10 passed (V2's 8 and the 2 of `media-setup`), the frozen check empty.
- **SAB gate:** no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm test`: Rust **461 passed**, 3 ignored, Vitest 92; `pnpm build` exit 0 (`offcut_core_bg` 713,306, `offcut_render_bg` 6,113,666 bytes, unchanged); the tracked-private check prints nothing.

**Found, not fixed.** None.

---

## 2026-10-19 - Prompt 84 (V3 24/30): rejection suite and E2E helpers

CI on Prompt 83's `f70f38c`: `ci` and `e2e-media` success.

**Done.**

- **`web/tests-e2e/helpers/fixtures.ts`** (§19.15), 604 lines:
  - `fixturesDir()`: `OFFCUT_FIXTURES_DIR`, else `fixtures/.gen` from the helper's own path.
  - `genFixture(name)`: the path, or an error ending "run pnpm fixtures" when the file or `STAMP` is absent, or when `STAMP.manifest_sha256` is not the SHA-256 of today's `fixtures/manifest.json` (hashed as the raw bytes, as `gen_fixtures.sh` does; they agree on D1).
  - `dropFile(page, file, name = "clip.mp4")`: V2's `dropClip` with a chosen name; `dropClip` now calls it.
  - `dropOversize(page)`, with `OVERSIZE_BYTES` (D-79): in the page, 501 references to one 1 MiB `Blob` in a `File` named `clip.mp4` (525,336,576 bytes), dropped in a `DataTransfer`.
  - `v3TestClip()`: `testclips/v3testclip.mp4` or `null`.
  - `readIndexedDb(page, store, key)`: the stored `Versioned` value; it never creates the database.
  - `rejectCodeOf(page)`: the panel's `data-reject`, or `null`.
  - Nothing reads a fixture while a file loads (V2 CL item 40).
- **`web/tests-e2e/rejections.spec.ts`** (§19.16, D-78, D-79, D-93), 208 lines, **16 cases, 15 `@no-gpu`**:
  - **Import-time refusals:** 12 come from one loop over the manifest's `rej_*` entries except `rej_silent.mp4` (read at load: the manifest is committed; the fixtures are read in the cases), plus the oversize case.
  - **Also:** `REJECT_NO_SPEECH` (untagged), "start over" and the `ok_vfr.mp4` accept case.
  - **Every case begins** as V2's do: `installFakeApi`, `routeAssets`, `page.clock.install()`, `/app`, the drop zone.
  - **Every refusal asserts:**
    - the code in `data-reject`;
    - the panel's heading, body and action from `messages.rejections`, with "500", "90", "1920" and "60" worked out from `LIMITS`;
    - neither the dropped name nor the code in the page's text;
    - after `flushAnalytics`, exactly one `clip_rejected` equal to `{ reject_reason: <code> }` and no `client_error`;
    - neither the dropped name nor the file's size in any event body;
    - `clips/` empty in OPFS.
  - `rej_vp9.webm` is dropped as `clip.webm`.
  - **The oversize case** also asserts that no request of the context carried a body of 64 KiB or more.
  - **The accept case:**
    - `clip_accepted` is `{ duration_bucket: "lt30", orientation: "landscape", source: "user" }`, with no `clip_rejected`;
    - `clips[<id>]` in IndexedDB has `schemaVersion` 1, its `clipId`, `sourceBytes` equal to the file's size, and `sourceHashPrefix` equal to the D-81 hash computed in Node;
    - no `clip.mp4` appears in the record.
  - **"Start over"** goes from a refusal (`rej_4k.mp4`) to the drop zone with `clips/` empty, and then takes `ok_vfr.mp4`.
- **`web/playwright.config.ts`**: `MEDIA_SUITES` gains `rejections.spec.ts`.

**Differs from the prompt, the guide or the plan.**

- **`REJECT_DECODE_UNSUPPORTED` asks Chrome before it runs.** S2 found Chrome on D1 decodes `avc1.f4001f` (entry of Prompt 63, `experiments.md`), so D-78's last branch applies. Rather than mark the case `test.fixme` for every machine, it calls `VideoDecoder.isConfigSupported` for that codec in the case's page and is `test.fixme` (with D-78's reason) only where the answer is `true`. On D1 it is skipped as fixme; a Chrome that refuses the file (the Windows runner, if so) runs the full assertions. Prompt 87 reads which on the runner.
- **`REJECT_NO_SPEECH` asserts one `clip_accepted`.** §19.16's "every rejection above: no `clip_accepted`" cannot hold for it: that clip is accepted at import and refused after the recognizer heard it (Prompt 81's step 9; Prompt 82 read `clip_accepted` then `clip_rejected`). The case asserts the truth, with a comment; the twelve loop cases and the oversize case assert no `clip_accepted`. "After Transcribing…" is read from a `MutationObserver` installed before load (V2 CL item 38).
- **The accept and "start over" cases wait for "Cleaning voice", not for the feed.** The feed shows from the import on, before `clip_accepted` is queued, so the first version flushed analytics too early and both cases failed ("received length 0"; only `landing_view` and `capability_check` were sent). "Cleaning voice" is pushed by the pipeline after `importClip` marks the clip accepted and queues the event, and needs no graphics processor (the media worker's audio and the chain in WASM).
- The prompt's `pnpm e2e:media --grep @no-gpu` "23 passed" reads **24 passed and 1 skipped** on D1 (the 2 of `media-setup`, V2's 8, these 15 less the fixme).

**Checked.**

- **The prompt's commands:** `pnpm e2e:media --list | grep -c rejections.spec.ts` 16; `--grep @no-gpu --list` "Total: 25 tests in 4 files" (23 and the 2 of `media-setup`).
- **The suite on D1:** the 12 loop cases 0.76 to 0.93 s each, the oversize case 0.73 s, "start over" 1.4 s, the accept case 1.2 s, `REJECT_NO_SPEECH` 14.3 s (`ensureModelCached` and one pass of the recognizer).
- **Drill:** with `rej_4k.mp4` moved out of `fixtures/.gen/`, its case failed "rej_4k.mp4 is not in …\fixtures\.gen: run pnpm fixtures"; moved back, `pnpm fixtures` says up to date.
- **`pnpm e2e:media --grep @no-gpu`:** 24 passed, 1 skipped (1.9 min). **`--grep NO_SPEECH`:** 3 passed (the case and the 2 of `media-setup`).
- **SAB gate:** no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `pnpm test`: Rust **461 passed**, 3 ignored, Vitest 92; `pnpm build` exit 0 (bundles unchanged: only tests changed); `pnpm e2e` 12 of 12; the frozen check and the tracked-private check print nothing.

**Found, not fixed.** None.

---

## 2026-10-19 - Outside a prompt: the Windows media job generates the fixtures (Prompt 87's `e2e-media.yml` part, brought forward)

**Why.** CI's `e2e-media` on `a47b3a2` (Prompt 84) failed at "Playwright, the media cases that need no graphics card": the job runs `pnpm e2e:media --grep @no-gpu`, which since Prompt 84 selects the rejection suite's 15 `@no-gpu` cases. 14 of them read generated fixtures through `genFixture`, and the job never ran `pnpm fixtures` (Prompt 87 was to add it), so they failed with "run pnpm fixtures", as written to. `ci` passed. The plan's order put the tag (Prompt 84) before the job could serve it (Prompt 87).

**Done.** `.github/workflows/e2e-media.yml`, as §18.6 and Prompt 87 give it for this file:

- **A first step**, "Check out with LF line endings": `git config --global core.autocrlf false` before the checkout. The Windows image checks text out with CRLF, and `sh` cannot run `fixtures/gen_fixtures.sh` so. **Open item 9 is closed** with its first option; no `.gitattributes`, so no new file.
- "Restore the model files and the reference clip" moves directly after "Install ffmpeg", followed by **"Generate the fixtures"** (`pnpm fixtures`). The script downloads the reference clip itself when the cache does not hold it; `VITE_ASSET_BASE_URL` is in the job's `env`.
- The head comment names the rejection suite. The Playwright command is unchanged (`--grep @no-gpu`, now 23 cases and the 2 of `media-setup`).

Nothing else changed: no app code and no test. Prompt 87 keeps its `ci.yml` work (the pinned Linux ffmpeg) and its guards; for `e2e-media.yml` it has this left to read (the `REJECT_DECODE_UNSUPPORTED` result on the runner, the job's time against open item 10).

**Checked.** The step order and indentation, read back. A YAML parser is not installed on D1 and none was added; the runner is the check, read below.

**Read on the runner:** `7d8980b` gave `ci` success and `e2e-media` success. The Windows job generated the fixtures (the LF checkout worked) and ran the 23 `@no-gpu` cases.

---

## 2026-10-19 - Prompt 85 (V3 25/30): pipeline and export cases for V3; full media suite on D1

CI on `7d8980b`: `ci` and `e2e-media` success (above).

**Done.**

- **`web/tests-e2e/pipeline-preview.spec.ts`, REOPENED CONTRACT (D-91).** 525 lines. The three cases that asserted V2's stand-ins are rewritten to §19.17; the other V2 cases keep their expectations.
  - **"Feed"**: "Cleaning voice" and "Transcribing…" are the first two lines in either order; then only "Found:" lines, in the order of the README's V3 expected feed. Lines are only ever added (V2's check, kept).
  - **"Real detections only"**: through `expectFeed`, every "Found:" line is one of the README's "Expected feed of V3" table, in its order, and the four rows not marked "(only if …)" are all there. "Found: brutal" (row 4, marked) is optional. The README's table is read by `expectedFeedV3`, which replaces V2's `expectedAmounts`.
  - **"Events"**: `stage_timing` is `probe_audio`, then `audio_chain` and `asr` in either order, then `detect_scene`, with `asr_backend` on `asr` only. `pipeline_done` has `n_number` 2, `n_list` 1, `n_from_to` 1 and `n_keyword` 0 or 1. The printed line also gives the chain's time.
  - **Extended, "OPFS"**: IndexedDB `clips`, `transcripts` and `edits` hold one record each (counted in one transaction). The clip's records are read with `readIndexedDb`, and `prosody.per_word.length` equals `transcript.words.length`.
  - **"Event shape"**: its allowed events gain `clip_rejected` (`REJECT_REASONS`); `audio_chain` was already in its stages. The feed text that must reach no request is now the V3 lines, "Cleaning voice" included.
  - **New, D1 only: "second recording"**, a test of its own after the group, so a failure in it hides none of the group's cases. Skipped with "testclips/v3testclip.mp4 is on D1 only" when the clip is absent. It drops the clip, waits for `ready`, and checks `clip_accepted` `{ source: "user", orientation: "landscape", duration_bucket: "lte90" }`. The feed is the README's "The V3 test clip as recognized" expected feed through `expectFeed`. The README says the KeywordPop may be absent: `expectedFeedOfTestClip` treats as optional the one line that names a word, not a quantity (no digit or currency sign), which is "Found: that's". Then the preview plays: two pictures a second apart differ, held in memory only.
- **`web/tests-e2e/export-creator.spec.ts`: new, D1 only, "no metadata" (D-100).** 408 lines, after the group; the eleven V2 cases are unchanged.
  - **The export:** drop `v3testclip.mp4`, wait for `ready` and for "Play" to be enabled (known issue 4), seed a Creator token, export, and save the file in a `mkdtemp` folder of the system temp directory. The context and the folder are removed in a `finally`.
  - **`metadataProblems(file, source)`** lists what it finds; the case expects `[]`. Checked:
    - `ffprobe -show_entries format_tags:stream_tags`: no tag key containing `location` or `iso6709`;
    - no `creation_time` but the epoch;
    - no `creation_time`, `title`, `comment` or `encoder` with the source's value;
    - a byte search for `©xyz` and `ISO6709`.
  - **Then** `verifyMp4(file, "creator", sourceDurationMs(source))`: "PASS 6", no "FAIL", exit 0.

**Differs from the prompt, the guide or the plan.**

- **The full run has 1 skipped, not 0.** It is `REJECT_DECODE_UNSUPPORTED`, which marks itself `test.fixme` because Chrome on D1 decodes the High 4:4:4 file (D-78; Prompt 84's entry). It is not the skip the prompt warns of (a missing test clip): both D1-only cases ran and passed.
- **The test clip's optional line is told by its content**, not by a mark in the README. That README paragraph says the KeywordPop may be absent, but marks no line. Today's rule (no digit and no currency sign) singles out "Found: that's" only.
- `pnpm e2e:media --grep "second recording\|no metadata"` was not run on its own: both cases ran in the full keyed run. The gate's build carries no key, under which the export case would fail at the entitlement.

**Checked.**

- **The list:** `pnpm e2e:media --list`: "Total: 50 tests in 5 files" (48 cases: V2's 30, the rejection suite's 16, these 2; and the 2 of `media-setup`).
- **Drill:** the metadata check, copied into a temporary `zz-metadata.spec.ts` and pointed at `testclips/v3testclip.mp4` itself, failed with 10 problems: `format: a location tag, location`, `format: a location tag, location-eng`, the source's `creation_time` (format and both streams), the source's `encoder` (stream 0), and the bytes of a `©xyz` atom. The copy and its config are deleted. No coordinate is written here.
- **The full suite on D1, keyed.** The test key was worked out from the seed of `fake-api.ts` in one shell (it equals `ci.yml`'s `E2E_TEST_PUBLIC_KEY`), exported there for one `pnpm build`, and unset after (`env` then holds it 0 times; it entered no file). `pnpm e2e:media`, detached: **49 passed, 1 skipped, 0 failed, 10.5 min**. Notes from the run:
  - The recognizer ran on `webgpu`: 58,169 ms, with the voice chain 1,273 ms.
  - The longest main-thread task while processing was 200 ms.
  - The Creator export: `render_encode` 24,871 ms.
  - "second recording" took 1.2 min and "no metadata" 2.0 min.
  - No `offcut-e2e-*` folder is left in the temp directory.
- **SAB gate**, on a fresh build without the key (the key's text is in no bundle file): no `zz` file, no test key, `pnpm fixtures` up to date, `pnpm check` exit 0, `pnpm test`: Rust **461 passed**, 3 ignored, Vitest 92; `pnpm build` exit 0 (bundles unchanged); `pnpm e2e` 12 of 12; the frozen check and the tracked-private check print nothing.

**Found, not fixed.** None. No V2 case other than the three of D-91 failed, so no app regression was found.


---

## 2026-10-19 - Prompt 86 (V3 26/30): bench `audio_chain` stage and file name; corpus fetcher, runner and project

CI on Prompt 85's `1c994d7`: `ci` success (7.1 min), `e2e-media` success (8.2 min); the deploys skipped.

**Done.**

- **`bench/device-bench.ts`, REOPENED V2 CONTRACT (D-91).**
  - `STAGES` has six entries: `probe_audio`, `audio_chain`, `asr`, `detect_scene`, `render_encode`, `mux`. A run that lacks one of them still fails at `wholeNumber`, as in V2.
  - `BENCH_CLIP` picks the clip from a table `CLIPS`, each looked up only when chosen:
    - `reference` (the default): `referenceClip()`, named by its file name as V2 named it;
    - `portrait`: `genFixture("speech/speech_60s_portrait.mp4")`, named `speech/speech_60s_portrait.mp4`;
    - `v3testclip`: `v3TestClip()`, refused with "BENCH_CLIP=v3testclip needs testclips/v3testclip.mp4, which is on D1 only" when absent.
  - Any other value is refused, naming the three. The result's `clip` is the table's name, never a path.
  - The result file is `bench/results/<device>-<date>-<clip>.json`, `<clip>` being the `BENCH_CLIP` key. The head comment and the test's title say so.
- **`corpus/fetch_corpus.sh`** (POSIX `sh`, LF):
  - Reads `corpus/manifest.json` with one `node -e` (id, SHA-256, owner per line).
  - Founder clips come from `testclips/` by a fixed `case`: `founder-01` from `speech_scriptA_landscape_720p.mp4`, `founder-02` from `v3testclip.mp4`. A lent clip comes from `$OFFCUT_CORPUS_DIR/<id>.<ext>`, the first such file that is not `.txt` or `.part`.
  - Each clip is copied to `corpus/.clips/<id>.mp4.part`, hashed with `sha256sum`, and renamed only on a match; a copy already there with the right hash is kept as it is.
  - Prints `<id> ok`, `<id> missing` or `<id> hash mismatch`, and nothing else of a clip (no path, no file name). No network call. Exit 1 only when no clip is ready, with "no corpus clip is ready" on stderr.
- **`corpus/run_corpus.ts`** (a Playwright file, 350 lines). One test per manifest clip present in `corpus/.clips/`, titled `corpus <mode>: <id>`, each in its own browser context.
  - **Modes**, by `CORPUS_MODE` (anything else is refused when the file loads):
    - `demux` (default): drop, then wait for accepted, refused or failed. "Accepted" is the "Cleaning voice" line, which the pipeline adds once `importClip` has accepted the clip (Prompt 84's entry). Writes `corpus/.results/<date>-demux.json`.
    - `export`: wait for `ready` and for "Play" (known issue 4), seed a Creator token, export, save the file in a `mkdtemp` folder of the system temp directory (removed in a `finally`), and run `verifyMp4(file, "creator", sourceDurationMs(clip))`. The six booleans are its `PASS n` lines. Writes `<date>-export.json`.
    - `wer`: wait for `ready`; read `transcripts[clipId]` in the page with `readIndexedDb` (the clip id is the one folder of `clips/` in OPFS); compare its words with `corpus/.clips/<id>.txt` by Levenshtein over words, split into substitutions, deletions and insertions. Writes `<date>-wer.json`, counts only. A clip without its `.txt` is skipped ("no reference text").
    - `transcripts`: the constant map `TRANSCRIPT_FILES` has two entries, `founder-01` → `null` (never written: `a01` stays in the written form; the test is skipped saying so) and `founder-02` → `fixtures/labeled/a02-v3testclip.json`. A clip not of `owner: "founder"`, or not in the map, is refused by a thrown error before its context is made, so nothing of it is read. The labeled file gets `words` (`text`, `startMs`, `endMs`, `confidence`) and `prosody` (`per_word` of `energy_z`, `pitch_z`), in the files' own layout (one line per word).
  - **Result files** hold ids, codes, booleans and numbers. A result entry replaces the same clip's entry of the same day and mode; it is written before the test asserts, so a mismatch is listed too.
  - **Each test asserts only** that the outcome is not `E_INTERNAL` or `E_WORKER_CRASH` and equals the manifest's `expect`. A failure's message is `<id>: <code>`, or `<id>: <fixed words>` for a wait that timed out (`quietly` replaces Playwright's own message, which can quote the page).
  - **The outcome** is read in the page as a state and a code, never as text:
    - the rejection panel's `data-reject`;
    - for a failure, the code where the page states it (the three model failures' titles, D-57), else `E_INTERNAL` when the page threw (how `runPipeline` surfaces an unexpected fault), else `"failed"`;
    - in `export` mode, an export's failure code from its `export_failed` event.
  - In `demux` mode an `expect: "REJECT_NO_SPEECH"` clip is followed to its end, with the model put on the device first; any other clip stops at its acceptance and needs no model.
- **`web/playwright.config.ts`**: project `corpus` as §18.3 gives it (`testDir: "../corpus"`, `testMatch: "run_corpus.ts"`, `dependencies: ["media-setup"]`, `timeout: MEDIA_TIMEOUT_MS * 2`, `fullyParallel: false`, `workers: 1`, `use: { ...MEDIA_USE, screenshot: "off", video: "off", trace: "off" }`).
- **`web/tests-e2e/tsconfig.json`**: `include` gains `"../../corpus"`.
- **`package.json`**: `"corpus": "sh corpus/fetch_corpus.sh && pnpm --filter web exec playwright test --project=corpus"`.
- TS §5 already listed both corpus files (V1), so `check-file-tree.mjs` needed nothing.

**Differs from the prompt, the guide or the plan.**

- **A failed test's `error-context.md` would have held the page's text (D-112).** Playwright 1.63 writes that file for every failed test. If the test fails while a context of it is open (a test timeout), it adds an ARIA snapshot of that context's first page, which for a corpus clip is the feed with its "Found:" lines. The project's `screenshot`, `video` and `trace` settings do not cover it. `run_corpus.ts` therefore does two things:
  - it sets `PLAYWRIGHT_NO_COPY_PROMPT=1` in its worker, the switch Playwright reads before taking that snapshot (`lib/index.js`, `_takePageSnapshot`);
  - an `afterEach` closes a test's pages before its context, so that no page is left to snapshot.

  Drill, on a copy of `founder-01` under a lent id with `--timeout 25000` in `export` mode:
  - with both in place, `error-context.md` held no "Page snapshot" and the error was "lent-99: no outcome in time";
  - with both taken out (temporarily), it held one, with two `listitem` lines of the feed.

  The content was counted, not printed; the file and the copy are deleted.
- **`transcripts` mode writes `words` and `prosody`, not `sentences`.** §17.6 and SCHEMA name `words` and `prosody`. SCHEMA says an `asr` file has `sentences: null`, and `precision_recall.rs` panics ("sentences and words do not match its source") on an `asr` file with both. The prompt's "`words`/`prosody`/`sentences`" is read as the three fields the map concerns, with `sentences` kept `null`.
- **The normalisation turns every other character into a space; it does not delete it.** §17.6 says "remove every character but…". SCHEMA's `say` normalisation, which calls itself the WER mode's, and `normalized()` of `precision_recall.rs`, make it a space, so "3-item" is two words and not "3item". Commas inside digit groups are removed first, as §17.6 says, so "$2,000" is "$2000".
- **A failure whose code the page does not show is recorded as `"failed"`.** The `Outcome` type is `"accept" | RejectReason | ErrorCode | "failed"`. The "failed" page shows a code only for the model's three failures, and V3 sends no `client_error` for a pipeline failure (V4). No app file was changed to show more. `expect` is never a failure, so such a clip fails its test either way.
- **The tsconfig entry is `"../../corpus"`, not `"../corpus"`.** The file is `web/tests-e2e/tsconfig.json`, and the prompt's path is relative to `web/`; `bench` is included the same way (`"../../bench"`).
- **"Both `tsc` programs see `corpus/`":** the second program (`tests-e2e/tsconfig.json`) includes `corpus/`, as it includes `bench/`. The app's program includes `src` alone (D-63 (a)), so a Node type never reaches app code. Both `tsc` runs of `pnpm check` pass.
- **`--list` prints "Total: 4 tests in 2 files"**: the 2 corpus cases and the 2 of `media-setup`, which `--list` always includes (as in Prompt 84's entry). The guide expects 2.
- In `export` mode `verifier` is present only for a clip that was exported.

**Checked.**

- **The prompt's commands:**
  - `sh corpus/fetch_corpus.sh` with `OFFCUT_CORPUS_DIR=$HOME/offcut-corpus` (empty): `founder-01 ok`, `founder-02 ok`, exit 0, 1.6 s; a second run keeps both copies.
  - `--project=corpus --list`: the two cases (above).
  - `git ls-files --eol corpus/fetch_corpus.sh`: `i/lf w/lf`.
  - `pnpm check`: exit 0 (gate below).
  - `git status --short corpus`: only the two new files.
- **The fetcher's other paths**, on a copy of the script and a made-up manifest in the scratchpad: a lent clip with the wrong hash → `hash mismatch`, no file left; an id with no file → `missing`; a lent `.webm` → `ok`, copied as `<id>.mp4`; a `.txt` beside it is not taken; no clip ready → exit 1; `OFFCUT_CORPUS_DIR` unset → the lent clips `missing`.
- **The `lent` refusal (failure-path test).** The manifest was given a fake `lent-99` entry (`owner: "lent"`, a copy of `founder-01` under that id), not committed. `CORPUS_MODE=transcripts ... --grep lent-99` failed with "lent-99: refused, transcripts mode takes the founder's clips of its map only". The MD5 of `a02-v3testclip.json` was unchanged, no `corpus/.results/` was made, and no page was opened. The manifest was restored and the copy deleted.
- **The labeled-file writer**, cut out of the runner into a scratch file and run with Node:
  - today's `a02-v3testclip.json` is written back byte for byte, and a second pass is identical;
  - Levenshtein counts match hand-worked cases (a substitution and an insertion; all deletions; empty reference; equal);
  - "We made $2,000 in revenue, 3-item list; 4.7%!" gives `we made $2000 in revenue 3 item list 4 7% done`.
- **The bench, shortened (`RUNS = 1` for one run, put back to 10).** A build with the test key (worked out from `ci.yml`'s public key, given to that one `pnpm build:vite` only; unset after), `BENCH_DEVICE=d1 BENCH_CLIP=portrait pnpm bench:device`: 3 passed, 4.7 min. It wrote `bench/results/d1-2026-10-19-portrait.json` with `clip` `speech/speech_60s_portrait.mp4`, `clip_duration_ms` 60,000, `asr_backend` `webgpu` and six stages. One run, so not a measurement: `probe_audio` 1,324 ms, `audio_chain` 3,116, `asr` 79,935, `detect_scene` 254, `render_encode` 37,643, `mux` 19; peak memory 994,987,537 bytes. The file was deleted (Prompt 88 commits the runs).
- **The four modes on the founder clips**, with the same keyed build (beyond the prompt's tests; every output since deleted):
  - `demux`: 2 passed, 3.2 s each, both `accept`, `match: true`.
  - `export`: 2 passed (2.8 and 3.3 min), verifier `[true x 6]` for both.
  - `wer` on `founder-01`, against a temporary `founder-01.txt` of Script A as written in `fixtures/speech/README.md`: `words_ref` 159, `words_hyp` 157, 13 substitutions, 3 deletions, 1 insertion, WER 0.1069. The script spells "three thousand", the recognizer writes "3,000": a smoke reading only, not E-10 (D-88). `founder-02` was skipped (no text).
  - `transcripts`: `founder-01` skipped (written form); `founder-02` wrote 159 words. The diff changed only the `words` and `prosody` lines, and `precision_recall.rs` read it ("31 files read, 0 skipped"). Then `a02` was restored from its copy: filling it is Prompt 88's. What the gate said with that data is under "Found, not fixed".
- **SAB gate**, on a fresh build without the key (`grep` finds the key in no file of `web/dist`):
  - no `zz` file, no test key in the environment;
  - `pnpm fixtures` up to date;
  - `pnpm check` exit 0;
  - `pnpm test`: Rust **461 passed**, 3 ignored; Vitest 92;
  - `pnpm build` exit 0 (bundles unchanged: no app code changed);
  - `pnpm e2e` 12 of 12;
  - the frozen check, the tracked-private check and the untracked-private check print nothing.

  Counts equal Prompt 85's.

**Pinned versions.** No dependency added.

**Found, not fixed.**

- **For Prompt 88: with `a02`'s recognized data the precision gate fails, overall precision 0.872 (under 0.90).**
  - Missed: "500 paying users" and "500 paying customers". Both are in the transcript (words 65-67, 82-84), with the adjective "paying" between the number and its noun.
  - False positives: NumberReveal on "three months" (words 72-73) and on "three to four months" (93-94), both at 0.80, which the labels call no events; and a KeywordPop on "that's" (word 152), at 0.84.
  - Also: the `$2,000` label reads "in revenue in" where "in revenue" was labeled (reported, not matched).
  - Prompt 88 fills `a02` and applies §20.3: labels are not edited (D-87).
