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

## Entry 3 - 2026-11-05: Setup, the baseline (overnight run)

**What.** This entry records the baseline that every later entry is compared with. It also opens the draft pull request. No code changed.

The release is built tonight by one agent working alone, under `docs/design/overnight-run.md`. The founder's authorisations of 2026-11-05 are quoted there (§3). In short:

- the agent takes the design decisions the plan leaves open;
- it gives the upload's go (P156);
- it does design review 3 with the rubric of `overnight-run.md` §5;
- it merges, runs the deploy checks and tags, under the gates of §7.

Wherever the plan says "stop and ask", the agent follows `overnight-run.md` §6 and records the case under Open items.

**Checks.**

- Branch `design-build` at `f334dbc`, on top of `8d94a06`. `origin/main..8d94a06` is one commit, V6's closing entry.
- Frozen check (guide Step 0.2): empty.

**Baseline on D1.**

| Check | Result |
|---|---|
| `pnpm check` | pass (`check-file-tree`: longest file `messages.ts`, 679 lines) |
| `cargo test --workspace` | 812 passed, 0 failed, 4 ignored |
| Vitest | 522 passed |
| `pnpm build` | `check-hosts` ok; app shell, gzip, **351.6 kB** (main chunk 163.6 kB, CSS 7.3 kB) |
| `pnpm e2e` | 45 cases: `landing.spec.ts` 18, `signin.spec.ts` 15, `checkout.spec.ts` 12 |
| Media floor | V6's close (V6 CL Entry 19): 106 passed and 1 skipped. Not re-run now |

**`pnpm e2e` on D1 depends on load.**

- With the default 6 workers (12 cores), the first run passed 41 of 45. The failures were three `checkout.spec.ts` cases and `signin.spec.ts`'s "two tabs: the link opened in a new tab signs the first in through the broadcast".
- Run alone, the three checkout cases passed. The "two tabs" case passed 3 of 3 with `--repeat-each=3`.
- With `--workers=4`, two full runs each passed 44 of 45. "Two tabs" failed both times: the first tab did not show `account-signed-in` within 5 s.
- The code is untouched, so this is a timing flake of D1 under parallel load, in a case this release does not touch until Part C (P157). CI runs on Linux with one retry.

**How every gate of this release reads `pnpm e2e`.**

- `pnpm e2e --workers=4`.
- Then any case that failed is run again alone. "Two tabs" is the one known to fail in the full run.
- The gate passes only if each case that failed passes alone, and it was one of the cases above.
- Each entry records both results.

**Not done yet.** Prompt 151 comes next.

## Entry 4 - 2026-11-05: P151, foundations (dependencies, copy, lint)

**What changed.**

- **Dependencies.** `gsap` 3.15.1 and `motion` 14.9.1, pinned exactly (`pnpm --filter web add -E`).
- **`web/src/copy/landing.ts` (new).** It holds `landingCopy` and `pricingCopy`.
  - It holds all the copy of §4 to §6: the eyebrows (timecodes made by a function from seconds), the 8 feature cards, the 12 "Coming soon" rows in two groups, the CTAs, the marquee, the clip labels, `playClip`, the final call and the footer.
  - Kept word for word: `supportedBrowsers`, `privacyLine`, `whatLeavesLink`, every price and plan feature, and `merchant`.
  - Reworded as P151 point 4 says:
    - `hero`: "Say it once. Post it finished." (F-1).
    - `subhead`: §6's draft.
    - `demoCaption`: D-324's words.
    - `pricing.notOnSale`: "Start on Free with a free account. Creator goes on sale soon." (F-2, option (a)).
    - `waitlistHeading` is replaced by `updatesHeading`, "Hear when the next features open.".
  - Numbers in sentences come from `LIMITS`, or from named constants for the long path's planned figures (20 minutes, 5 to 10 shorts).
- **`messages.ts`.**
  - It imports both: `landing: landingCopy`, `pricing: pricingCopy`. No caller changed.
  - `notifyMe.label` is now "Email me when new features open" (D-328).
  - It went from 679 to 652 lines.
- **`LandingPage.tsx`.** One line: the waitlist heading reads `updatesHeading`.
- **`eslint.config.js`.**
  - `LANDING_CLIPS`, `LandingClip` and `LandingSlot` are added to the `ui` → `net/asset-fetch.ts` names.
  - D-319: `no-restricted-imports` refuses `gsap`, `gsap/*`, `motion` and `motion/*` everywhere in `src/` except `src/ui/landing/**`.
  - ESLint replaces a rule's options instead of adding to them. So the rule now holds both restrictions, the speech runtime's and D-319's, through a small `restrictedImports()` helper. The landing folder keeps the speech runtime's.
  - `workers/asr/whisper-runtime.ts` had the rule `off`. It now keeps D-319's half, so that file still may not import GSAP or Motion. This is stricter than before and changes nothing else.
- **TS.** §5 has a row for every new file of §10.1: `copy/landing.ts`, the `ui/landing/` folder and its 18 files, the two CSS modules, the JetBrains Mono subset, the posters folder and `public/og.png`. INV-15 now reads "`copy/messages.ts` and the files it imports".

**GSAP's licence, as read.** The package ships no licence text, only a pointer:

- `package.json`: "Standard 'no charge' license: https://gsap.com/standard-license."
- Each file's banner: "@license Copyright 2008-2026, GreenSock. All rights reserved. Subject to the terms at https://gsap.com/standard-license".

The text at that address was read on 2026-11-05. Quoted:

- **Grant (§II):** "Subject to the terms and conditions of this GSAP License, Webflow grants you a non-exclusive, worldwide license to use, reproduce, display, and implement GSAP Products solely for Permitted Uses."
- **Permitted Uses:** "the implementation and/or use of GSAP Products on any website, web application, or digital interface … by any person or entity … (which may include, for clarity, those of companies that compete with Webflow in other areas of business)".
- **Prohibited Uses:** "any implementation and/or use of GSAP Products in tools that allow users to build visual animations without code … that encourages, induces, or materially assists in creating a solution that competes with Webflow's visual animation building capabilities."
- **Restrictions (§III):**
  - no Prohibited Uses without consent;
  - no reverse engineering "for the purpose of creating Competitive Products";
  - no removing or altering "proprietary notices or branding".
- **Also in the text:** termination (stop using it and destroy copies) and amendments (a later version is used only under the revised licence).

**The reading.** The one restriction on use is the Webflow-competitor one, which §10.2 describes. The other clauses are ordinary licence terms, and Offcut keeps them:

- GSAP is used on a marketing page, not in a tool for building animations;
- Offcut does not reverse engineer GSAP;
- the bundle keeps GSAP's banner comments.

So F-8 stands, under `overnight-run.md` §6's test. **For the founder:** this reading is the agent's. If you read the other clauses as "more than the Webflow restriction", P155's GSAP work can be replaced by Motion and CSS, as §6 of that file says.

**What differs from the plan.** One thing: whisper-runtime.ts's rule, above.

**Checks (Tier 2).**

| Check | Result |
|---|---|
| `pnpm check` | pass. `check-file-tree`: 491 files; longest is `messages.ts`, 652 lines |
| Vitest | 522 passed |
| `pnpm build` | `check-hosts` ok; app shell, gzip, **354.0 kB** (main chunk 165.9 kB, up 2.3 kB for the copy). Neither library is in the bundle yet: nothing imports them |
| `pnpm e2e --workers=4` | 45 of 45 passed, all 18 landing cases unchanged ("two tabs" passed this time) |
| D-319 lint | A deliberate `import { gsap } from "gsap"` in `ui/pages/LandingPage.tsx` failed lint with D-319's message. The same import in a probe file under `ui/landing/` passed. Both were removed |
| `git grep -n "not open yet" web/src` | one hit: the terms' "parts of it are not open yet", which stays |
| Frozen check | empty |

**Open.** Nothing new.

**Found, not fixed.** None.
