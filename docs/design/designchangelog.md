# designchangelog.md - the design release, change by change

This file covers the design release between V6 and the cloud-transcription release. Its plan is `designimplementation.md`. Every change of the release is summarised here, in the same commit as the change.

## Open items

| # | Item | Who | Where |
|---|---|---|---|
| 1 | F-1: the H1 (recommended: "Say it once. Post it finished.") | Founder | §18 |
| 2 | F-2: Creator's line on the page; resubmit to Dodo (recommended: option (a), "Creator goes on sale soon.") | Founder | §13, V6 CL open item 10 |
| 3 | F-3: hide the in-app offers until live mode, as a small change of its own (V6 CL open item 17) | Founder | §18 |
| 4 | F-4: the caption option, from rendered stills (recommended: A, bottom-hug) | Founder, at G-3 | §14 |
| 5 | F-5: Part C, the other pages on the dark system (recommended: yes) | Founder | §15 |
| 6 | F-6, F-7, F-8, F-9: the accent font, the open-graph image, GSAP's licence, the optional clips | Founder | §18 |
| 7 | The clips, made with `remotion-prompts.md` and checked by §9.2 (G-1) | Founder | §9 |

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
