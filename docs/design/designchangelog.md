# designchangelog.md - the design release, change by change

This file covers the design release between V6 and the cloud-transcription release. Its plan is `designimplementation.md`. Every change of the release is summarised here, in the same commit as the change.

## Open items

| # | Item | Who | Where |
|---|---|---|---|
| 1 | F-3: hide the in-app offers until live mode, as a small change of its own after this release (V6 CL open item 17) | Founder | §18 |
| 2 | The upload of the seven re-encoded clips needs the founder's go (P156) | Founder | guide Step 6.3 |
| 3 | Design review 3 (P158) | Founder | guide Phase 8 |
| 4 | The Dodo resubmission and the tag `design` (P160) | Founder | guide Phase 10 |

## Entry 1 - 2026-11-04: the plan

**What.** This entry adds three files:

- `designimplementation.md`: the plan of the release, with decisions D-316 to D-335.
- `remotion-prompts.md`: the seven clips' prompts for Claude Code, run in a separate Remotion project.
- This changelog.

The branch `design-build` starts at `8d94a06` (V6's closing entry on `v6-build`, which is `origin/main` plus that one docs commit).

**Read before writing:**

- the landing page and everything it renders: `LandingPage.tsx`, `DropZone.tsx`, `NotifyMeForm.tsx`, `UnsupportedPage.tsx`, `CapabilityGate.tsx`, `SettingsPage.tsx`, `routes.tsx`, `main.tsx`;
- its copy (`messages.ts`, 679 of 700 lines), styles (`pages.module.css`, `components.module.css`, `tokens.css`), `asset-fetch.ts` and `pricing.ts`;
- `vercel.json`'s security policy, `vite.config.ts`, `eslint.config.js`, `check-hosts.mjs` (the app shell today: 351.6 kB) and `check-file-tree.mjs`;
- `landing.spec.ts` (18 cases) and every other spec's use of the drop zone;
- the caption placement (`placement.rs`, `safe_area.rs`, `captions.rs`, `styles.rs`) and the verifier's watermark mask;
- TS §5, §13, §39.3, §41; PS §9; BP §0.

**Founder's choices of 2026-11-04 recorded:**

- GSAP and Motion on the landing page.
- The app-shell budget raised "as much as needed": set at 500 kB (D-320).
- A unique page with no reference sites (D-317, "the page is a short").
- No logic changes (§2, §11).
- Every current feature named, and every post-V6 feature marked "Coming soon" (§4, §5).
- The hand-made demo video removed (D-324).

**Found while reading:**

- `landing.spec.ts` locates eight strings by substring, so the new page must hold each exactly once (D-334). Without that rule, a repeated privacy sentence or a second "What leaves your device" link would fail four existing cases.
- The landing's DropZone and NotifyMeForm classes are shared with `SignInDialog` and the unsupported page. They are restyled from the landing's own scope, not edited (§11).
- The Bottom captions sit centred at 68% of the frame's height. The Free watermark's place is read by the verifier's fixed mask, so lowering the captions by moving the safe area would change a protected proof. Option A of §14 avoids that.
- TS §5 lists `copy/messages.test.ts`, which the repository does not have. This is not this release's to fix; it is noted for the copy-back.

**Not done yet.** No code. The build guide and the coding prompts follow the founder's approval of this plan.

## Entry 2 - 2026-11-05: the founder's answers, the clips read, the guide and the prompts

**What.** This entry adds two files:

- `designbuildguide.md`: the phases, commands, tiers and frozen check.
- `coding-promptsdesign.md`: the SAB, Setup and Prompts 151-160.

`designimplementation.md` changes in five places:

- §18 records the answers.
- §9.1 records the clip read.
- §16 now names the prompts.
- L-8, L-9 and L-11 are made testable (L-8 reads the `.js` bodies for `ScrollTrigger`; L-9 navigates in the app rather than dropping a file; L-11 uses the existing `axe(page)` helper, serious and critical).
- D-333's review is P158.

**The founder's answers (2026-11-05: "do what you recommend").** F-1 to F-9 are all as recommended in §18. Open items 1, 2 and 4 to 7 of Entry 1 are closed.

**The clips (G-1), read from `offcut-remotion/out/`.** All seven web files, masters and posters exist.

| Clip | Web size | Frames | Bytes | Cap |
|---|---|---|---|---|
| hero | 720 × 1280 | 300 | 773,591 | 2,500,000 |
| before-after | 1280 × 720 | 360 | 486,785 | 3,000,000 |
| caption-styles | 720 × 1280 | 270 | 360,621 | 2,000,000 |
| visuals | 960 × 960 | 360 | 364,094 | 2,500,000 |
| how-it-works | 1280 × 720 | 360 | 992,038 | 2,500,000 |
| privacy | 1280 × 720 | 180 | 50,691 | 1,500,000 |
| coming-soon | 1280 × 720 | 240 | 269,786 | 1,500,000 |

- **What passes.** Each clip is H.264 High at 30 fps, with no audio stream, faststart, and is under its cap. Every first frame matches the last (six-frame contact sheets read).
- **What fails.** Every file is `yuvj420p` (full range), not `yuv420p`. P156 re-encodes from the masters before the upload (guide Step 6.1).
- **Truth rules.** They hold. No other company's name or UI appears. "Coming soon" stays on screen for the whole coming-soon clip. how-it-works uses the app's real screenshots.
- **Footage.** `public/footage/` was never filled, so the other clips show a drawn silhouette in place of the founder. That is allowed; a later re-render is a new upload.

**Not done yet.** No code. Setup comes next, and its baseline is Entry 3.
