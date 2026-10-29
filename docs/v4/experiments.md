# experiments.md - outcome of every reading V4 takes (v4implementation §21)

Every reading is taken on D1 and recorded under the label `d1` (D-114). One section per row of §21.1 and one per decision of §21.2, each filled when its reading is taken (Prompt 126 takes most of them; the step that takes each is in §21.1).

## Edit to preview
Question: Does a word fixed in the transcript reach the preview within 150 ms (the 90th percentile; TS §13.5, D-173)?
Method: `web/tests-e2e/preview-sync.spec.ts`, its case "edit to preview": on the paused 60 s portrait fixture, 20 word commits, each timed from Enter to "Saving…" (which the update rule shows after `setScene` has answered, and `setScene` answers after the redraw; V4 changelog Entry 33). Five runs of the suite, on `pnpm build:e2e` with the test key, D1 (Chrome, the GPU of D1), commit `f4a14f7` plus Prompt 126's measurement build.
Numbers: d1 — 90th percentile 25, 26, 29, 27, 29 ms (runs 1-5); medians 20-23 ms; the worst single commit 34 ms. The two full media runs of Entry 37 read 27 and 25 ms.
Date: 2026-10-29
Decision: Met, with a margin of five: the scene build of D-173 needs no split per chunk. A pass on D1 is not a pass on R1 (D-114); V9 reads it again.

## Style thumbnails
Question: Do the Style tab's three live cards (Clean, Bold, Mono of the paused frame) appear within 400 ms of opening the tab (D-186, assumption)?
Method: A temporary measurement spec (deleted after, V4 changelog Entry 38) on the paused reference clip: the Style tab opened 10 times from the Captions tab, each timed in the page from the tab's `pointerdown` to the third card's canvas (the same timing as `review-edit.spec.ts` case 14).
Numbers: d1 — 81, 28, 33, 24, 28, 23, 24, 25, 18, 29 ms; the first opening is the slowest (the render worker's first `styleThumbnails`), the others 18-33 ms. Case 14 of the full media runs read 62 and 56 ms (Entry 34).
Date: 2026-10-29
Decision: Met. The thumbnail size stays as built (the preview profile, D-186); no card is rendered one at a time.

## Hit test and hover
Question: Does `hitTest` answer within one frame (16 ms) while the preview is paused (D-174)?
Method: A temporary `performance.measure` around `pool.render.hitTest` in `usecases/preview-interact.ts`'s `hitAt` (in a measurement build only, removed after) and a temporary spec that moved the pointer to 100 places of the paused reference clip's preview, waiting for each answer.
Numbers: d1 — 100 calls: median 0.5 ms, 90th percentile 0.7 ms, worst 7.6 ms; none over 16 ms.
Date: 2026-10-29
Decision: Met. Hover hit tests every frame (not every second frame).

## App shell with the font
Question: Is the app shell (HTML, JS and CSS, without WASM) within 400 kB gzip with the self-hosted interface font (TS §13.5, D-180)?
Method: `scripts/check-hosts.mjs`'s bundle size, printed by `pnpm build` (the production build).
Numbers: d1 — 337.2 kB gzip (index JS 149.8 kB, the ASR worker 146.4 kB, render worker 19.6 kB, media worker 8.0 kB, CSS 6.9 kB, audio worker 6.1 kB, HTML 0.3 kB). The font itself, `inter-latin-<hash>.woff2`, is an asset fetched after the shell. At V4's baseline (Entry 1, before the font and the editor) it was 307.3 kB: V4 added 29.9 kB.
Date: 2026-10-29
Decision: Met. D-180's contingency (system fonts) is not taken.

## Preview drift and late frames
Question: Does the preview keep the frame within 80 ms of the sound over a whole 60 s play, with at most 5% of frames late (TS §20, PS §20.1)?
Method: `preview-sync.spec.ts`'s case "drift": the 60 s portrait fixture played to its end; the player's `data-max-drift-ms`, `data-late` and `data-frames` (D-155). Five runs; the worst counts.
Numbers: d1 — max drift 22.9, 23.6, 22.2, 26.9, 24.2 ms; 0 late frames of 1,800 in every run. The two full media runs of Entry 37 read 25.2 and 19.0 ms. The 5 s play of `pipeline-preview.spec.ts` draws 152-153 frames, 0 late.
Date: 2026-10-29
Decision: Met: worst 26.9 ms, 0% late. BP §6.2 row 1's contingency (a rendered preview file) is not needed (below).

## First preview frame
Question: Is a frame of the clip on the canvas within 500 ms of `ready` (assumption, M1.5)?
Method: `preview-sync.spec.ts`'s case "first frame": from the canvas's insertion (the clip `ready`) to the play button's enabling, which follows `attachPreview`'s answer, given once the first frame is drawn; one screenshot proves the canvas shows a frame of the clip.
Numbers: d1 — 93, 76, 94, 85, 85 ms. Before Prompt 124's two fixes it was 316-420 ms (Entry 33); with the first fix alone it was 680-700 ms in two loads of three (the decoder's cold start, Entry 35).
Date: 2026-10-29
Decision: Met. As built: the page keeps one `AudioContext`, made at the first play, and the render worker decodes the first frame while the scene is set (Entries 35, 37).

## Seek to first frame
Question: Paused, does a seek show its frame within 300 ms (D-148, assumption, M1.5)?
Method: `preview-sync.spec.ts`'s case "seek": paused, a seek to 55.0 s, the fixture's white frame; timed until the canvas's mean brightness rises above 200.
Numbers: d1 — 89, 80, 94, 94, 78 ms (mean brightness 235 each time). The full media runs of Entry 37 read 60 and 88 ms.
Date: 2026-10-29
Decision: Met. The spinner after 300 ms (D-148) stays for slower machines.

## Downscale rule
Question: Does D1 keep the preview at full size (`data-downscaled` false after a 60 s play), as the downscale rule of D-149 intends for a machine of its class?
Method: `preview-sync.spec.ts`'s drift case reads `data-downscaled` after the whole clip has played.
Numbers: d1 — `false` in all five runs and in both full media runs.
Date: 2026-10-29
Decision: As expected on D1: not downscaled. R1 may differ (D-114); V9 reads it there.

## build_scene and frame_at
Question: Does `build_scene` take at most 1 s and `frame_at` at most 2 ms on average, natively, for a clip of the reference length (D-166, TS §19)?
Method: `scene_timing`, an ignored test added to `crates/offcut-scene/tests/determinism.rs` (Prompt 126): `cargo test -p offcut-scene --release --test determinism -- --ignored scene_timing --nocapture`. The reference clip's labeled transcript (`a01-scriptA`, of which the 60 s portrait fixture is the start: 2,013 frames) detected and validated by the test helpers, then `build_scene` timed, then `frame_at` over every frame; the other five labeled scenes printed beside it. Validation runs in the helper before the timed build.
Numbers: d1 — a01-scriptA: `build_scene` 6.3 ms, `frame_at` mean 0.002 ms over 2,013 frames. The other five: `build_scene` 0.6-1.5 ms, `frame_at` mean 0.001-0.002 ms.
Date: 2026-10-29
Decision: Met by far: no profiling needed. In the browser the whole `detect_scene` stage, which holds the build, is read by the bench (next).

## detect_scene and render_encode
Question: In the browser on D1, is `detect_scene` within 10% of V3's plus about 30 ms for the plan and the time map, and `render_encode` at most 90 s per 60 s (E-4)?
Method: `BENCH_DEVICE=d1 BENCH_CLIP=portrait pnpm bench:device` (one warm-up, ten measured runs of the 60 s portrait fixture, headed Chrome), on a keyed production build of `f4a14f7`; written to `bench/results/d1-2026-10-29-portrait.json` and compared with V3's `bench/results/d1-2026-10-19-portrait.json`.
Numbers: d1 — medians, V3 → V4: `detect_scene` 195 → 300.5 ms (p90 227 → 365); `render_encode` 30,848 → 38,773 ms (p90 31,209 → 40,751); `mux` 16 → 25 ms. In the same runs the stages V4 did not change were slower too: `probe_audio` 875 → 1,581 ms, `audio_chain` 1,963 → 4,094 ms, `asr` 69,469 → 98,272 ms; total 102,690 → 140,494 ms. The machine was loaded (Docker Desktop's virtual machine, left running after Prompt 125's container drill, the editor and its language server) and settled during the bench: the last four runs read `detect_scene` 213-249 ms, `asr` 83-84 s, totals 119-121 s.
Date: 2026-10-29
Decision: `render_encode` is met with a wide margin (38.8 s per 60 s, against 90 s; BP §6.2 row 2's line of 150 s is far away). `detect_scene` misses its line (about 245 ms) on this run, by a margin no larger than the slowdown of the stages V4 left alone (asr +41%, probe_audio +81%, against detect_scene +54%), and is within it in the run's last four (213-249 ms). Recorded as missed under load, not as a regression of V4's code: the native `build_scene` takes 6.3 ms (above). A run on a quiet D1 (nothing else running) re-reads it in V5, beside V5's own bench; V9 reads R1.

## Golden similarity across adapters
Question: Do golden frames written on D1's software adapter (WARP) read the same on the CI runner's (lavapipe), at the SSIM of 0.98 every golden must reach (D-135, §21.1)?
Method: `crates/offcut-render/tests/golden_frames.rs` (V4 Prompt 101). The two video-only goldens, `crop_landscape_050` (a 1920 x 1080 synthetic source, the crop at offset 0.5) and `punch_in_110` (a 1080 x 1920 synthetic source, the centre 1/1.1), drawn by `Renderer::render_to_image` at 540 x 960 and written on D1 with `OFFCUT_GOLDEN_UPDATE=1`, then compared on each adapter: SSIM on BT.709 luma over 8 x 8 windows (`tests/common/ssim.rs`). D1: Microsoft Basic Render Driver (WARP), DX12, with the Windows SDK's `dxcompiler.dll` 1.8.2502.11 (v4changelog known issue 3). Runner: `ubuntu-24.04` image 20261012.334.1, llvmpipe (LLVM 20.1.2, 256 bits), Vulkan, `mesa-vulkan-drivers` 25.2.8-0ubuntu0.24.04.4 installed by the Linux job; the log line `ssim <name> = … on <adapter>` of the step "Golden frames, with their readings" (run 39825539363, commit `1623d88`).
Numbers: crop_landscape_050 is 1.0000 on d1 WARP and 1.0000 on lavapipe; punch_in_110 is 1.0000 on d1 WARP and 0.9999 on lavapipe.
Date: 2026-10-22
Decision: Both at least 0.98 on both adapters: one folder, `fixtures/golden/`, serves both; no per-adapter folders. The threshold stays 0.98. The two goldens hold a `Video` command only; the other fourteen (Prompt 107) draw glyphs through Vello, and are read on both adapters again then.

Prompt 107, the other fourteen and the parity case (2026-10-23):
- **The cases.** Each style with each kind's visual, drawn from `offcut-scene`'s snapshot inputs at their "during" instant (`tech_list` on a source turned 90°), and Clean's NumberReveal with and without the watermark; all written on D1 with `OFFCUT_GOLDEN_UPDATE=1`, then compared.
- **The parity case** draws each style's from-to frame at 1080 x 1920, box-filters it to 540 x 960 (2 x 2 mean), and compares it with the same frame drawn at 540 x 960, at SSIM >= 0.99 (D-172).
- **D1 numbers.** Every golden is 1.0000 on d1 WARP against itself on the rerun. Parity is 0.9996 (Clean), 0.9997 (Bold), 0.9998 (Tech) on d1 WARP.
- **Stability.** WARP is not byte-stable: two writings differed in 1 to 7 pixels by one level in three goldens, at SSIM 1.0000.
- **Lavapipe** (llvmpipe, LLVM 20.1.2, 256 bits; CI run 39975720726 on `248a44a`, step "Golden frames, with their readings"):
  - Goldens: `bold_from_to` 0.9994, `bold_keyword` 0.9994, `bold_list` 0.9994, `bold_number` 0.9995, `clean_from_to` 0.9995, `clean_keyword` 0.9994, `clean_list` 0.9994, `clean_number` 0.9995, `clean_number_nowatermark` 0.9995, `clean_number_watermark` 0.9995, `tech_from_to` 0.9994, `tech_keyword` 0.9995, `tech_list` 0.9997, `tech_number` 0.9995; `crop_landscape_050` 1.0000 and `punch_in_110` 0.9999 as in Prompt 101.
  - Parity: Clean 0.9994, Bold 0.9995, Tech 0.9995.
  - 19 passed.
- **Decision:** every golden is at least 0.98 and every parity case at least 0.99 on both adapters, with WARP and lavapipe at least 0.9994 apart from 1.0. One folder, `fixtures/golden/` (16 PNGs), still serves both. The thresholds stay 0.98 and 0.99.

Prompts 125 and 126, the finished build (2026-10-29):
- **Lavapipe** (CI run 40844706822 on `f59a610`, the Linux job; `mesa-vulkan-drivers` 25.2.8-0ubuntu0.24.04.4, llvmpipe LLVM 20.1.2, 256 bits): `golden_frames` 19 passed, in `cargo test` and again in the step with the readings; e.g. `clean_number` 0.9995, parity Clean 0.9994. Also, with no Mesa (an `ubuntu:24.04` container on D1, Prompt 125's drill): the 17 adapter cases fail "no software adapter"; with it, 19 passed on llvmpipe at 0.9994-1.0000.
- **d1 WARP**: the 16 goldens at 1.0000; parity Clean 0.9996, Bold 0.9997, Tech 0.9998; 19 passed.
- **Decision:** unchanged: one folder serves both adapters, at 0.98 and 0.99.

## offcut_render.wasm size
Question: How much did the plan crate and V4's scene make the render bundle grow (§21.1; over 300 kB growth is investigated)?
Method: The size of `web/src/wasm/pkg/render/offcut_render_bg.wasm` after `pnpm build:wasm`, at V4's baseline (Entry 1), after the plan crate (Prompt 98, Entry 8) and now.
Numbers: d1 — 6,113,666 bytes at the baseline; 6,287,309 after the plan crate (+173,643: the validator, the rules producer, the plan and time-map types' serde, the font registry); 6,361,436 now (+247,770 from the baseline). `offcut_core_bg.wasm`: 713,306 → 743,885 bytes (+30,579, the time map and `assemble`).
Date: 2026-10-29
Decision: Under the 300 kB line: nothing to investigate. `serde_json` is not in the render bundle (feature `prompt` off, D-122).

## Decision: preview drift contingency (BP §6.2 row 1)
Question: Does the preview's drift or its late frames make BP §6.2 row 1's contingency (a 360 x 640 preview file rendered with the export loop and played in a `<video>`) necessary?
Method: The readings "Preview drift and late frames", "First preview frame" and "Seek to first frame" above.
Numbers: d1 — worst drift 26.9 ms over 60 s (target 80), 0% late frames (target 5%), first frame 76-94 ms, seek 78-94 ms; never downscaled.
Date: 2026-10-29
Decision: Drift within budget: no preview-file contingency. The live preview of TS §20 stays. The row is a decision on R1 in BP §6.2: V9 reads it there (D-114).

## Decision: Vello in a worker contingency (BP §6.2 row 2)
Question: Is Vello in the render worker reliable enough that BP §6.2 row 2's contingency (a `Canvas2dBackend` drawing the same `DisplayList`, with goldens of its own) is not needed?
Method: Every run of V4 on D1: the media suites (two full runs of 78 in Entry 35, two in Entry 37, one in Prompt 127), five `preview-sync` runs, the ten-run bench; the goldens on WARP (D1) and lavapipe (CI).
Numbers: d1 — no GPU loss, no worker crash, no failed frame outside the faults the tests cause on purpose (`crash`, `hangWorker`, `failNextSetScene`); 16 goldens at 1.0000 on WARP and at least 0.9994 on lavapipe; `render_encode` 38.8 s per 60 s.
Date: 2026-10-29
Decision: Vello in a worker reliable: no Canvas2D backend. The display list stays serializable, so the seam is still there.
