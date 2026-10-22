# experiments.md - outcome of every reading V4 takes (v4implementation §21)

Every reading is taken on D1 and recorded under the label `d1` (D-114). One section per row of §21.1 and one per decision of §21.2, each filled when its reading is taken (Prompt 126 takes most of them; the step that takes each is in §21.1).

## Edit to preview
Question:
Method:
Numbers:
Date:
Decision:

## Style thumbnails
Question:
Method:
Numbers:
Date:
Decision:

## Hit test and hover
Question:
Method:
Numbers:
Date:
Decision:

## App shell with the font
Question:
Method:
Numbers:
Date:
Decision:

## Preview drift and late frames
Question:
Method:
Numbers:
Date:
Decision:

## First preview frame
Question:
Method:
Numbers:
Date:
Decision:

## Seek to first frame
Question:
Method:
Numbers:
Date:
Decision:

## Downscale rule
Question:
Method:
Numbers:
Date:
Decision:

## build_scene and frame_at
Question:
Method:
Numbers:
Date:
Decision:

## detect_scene and render_encode
Question:
Method:
Numbers:
Date:
Decision:

## Golden similarity across adapters
Question: Do golden frames written on D1's software adapter (WARP) read the same on the CI runner's (lavapipe), at the SSIM of 0.98 every golden must reach (D-135, §21.1)?
Method: `crates/offcut-render/tests/golden_frames.rs` (V4 Prompt 101). The two video-only goldens, `crop_landscape_050` (a 1920 x 1080 synthetic source, the crop at offset 0.5) and `punch_in_110` (a 1080 x 1920 synthetic source, the centre 1/1.1), drawn by `Renderer::render_to_image` at 540 x 960 and written on D1 with `OFFCUT_GOLDEN_UPDATE=1`, then compared on each adapter: SSIM on BT.709 luma over 8 x 8 windows (`tests/common/ssim.rs`). D1: Microsoft Basic Render Driver (WARP), DX12, with the Windows SDK's `dxcompiler.dll` 1.8.2502.11 (v4changelog known issue 3). Runner: `ubuntu-24.04` image 20261012.334.1, llvmpipe (LLVM 20.1.2, 256 bits), Vulkan, `mesa-vulkan-drivers` 25.2.8-0ubuntu0.24.04.4 installed by the Linux job; the log line `ssim <name> = … on <adapter>` of the step "Golden frames, with their readings" (run 39825539363, commit `1623d88`).
Numbers: d1 WARP: crop_landscape_050 1.0000, punch_in_110 1.0000. lavapipe: crop_landscape_050 1.0000, punch_in_110 0.9999.
Date: 2026-10-22
Decision: Both at least 0.98 on both adapters: one folder, `fixtures/golden/`, serves both; no per-adapter folders. The threshold stays 0.98. The two goldens hold a `Video` command only; the other fourteen (Prompt 107) draw glyphs through Vello, and are read on both adapters again then.

## offcut_render.wasm size
Question:
Method:
Numbers:
Date:
Decision:

## Decision: preview drift contingency (BP §6.2 row 1)
Question:
Method:
Numbers:
Date:
Decision:

## Decision: Vello in a worker contingency (BP §6.2 row 2)
Question:
Method:
Numbers:
Date:
Decision:
