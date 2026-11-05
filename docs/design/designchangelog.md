# designchangelog.md - the design release, change by change

This file covers the design release between V6 and the cloud-transcription release. Its plan is `designimplementation.md`. Every change of the release is summarised here, in the same commit as the change.

## Open items

| # | Item | Who | Where |
|---|---|---|---|
| 1 | F-3: hide the in-app offers until live mode, as a small change of its own after this release (V6 CL open item 17) | Founder | §18 |
| 2 | ~~The upload of the seven re-encoded clips~~: **closed** 2026-11-05, uploaded under `overnight-run.md` §3.2 | Agent | Entry 9 |
| 3 | Design review 3 (P158): done as a self-review under `overnight-run.md` §5, rubric passed in round 2. The founder reads `~/design-review/` and `copy/landing.ts` in the morning | Founder | Entry 11 |
| 4 | The Dodo resubmission (V6 CL open item 10). The tag `design` is done (Entry 13) | Founder | guide Phase 10 |
| 5 | GSAP's `@license` banners are not kept in the minified bundle. The notice is in `web/LICENSES.md` and the package. Keeping it in the bundle needs a `vite.config.ts` option (outside this release's files). Decide whether that is needed | Founder | Entry 8 |
| 6 | ~~Part B (DC-13)~~: **closed** 2026-11-05. The founder chose (a) with the captions "a little bit low"; the positioning review chose half of A's movement, below the middle only; the helper's declared change; `review-edit` 16 of 16 | Founder, agent | Entry 16 |

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
- the bundle was expected to keep GSAP's banner comments. **Corrected in Entry 8:** the minified build does not, and this is open item 5.

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

## Entry 5 - 2026-11-05: P152, the skeleton of the new page

**What changed.**

- **`tokens.css`.**
  - The `@font-face` "JetBrains Mono UI" (weight 700, `swap`).
  - D-316's rule `:root:has([data-theme="dark"]) body { background: #0e0e11 }`.
  - The header comment and the dark section's comment now say the landing page uses Inter and the dark system.
- **The font.** `web/src/ui/assets/jetbrains-mono-latin.woff2`, made as guide Phase 2 says:
  - `varLib.instancer` with `wght=700`, then `pyftsubset` with the unicodes of the guide, WOFF2, no layout features;
  - 99 code points, 104 glyphs, no `fvar`;
  - 4,912 bytes, SHA-256 `146e9dd8…c2ee`, recorded in `web/LICENSES.md`.
  - The build emits it as `assets/jetbrains-mono-latin-<hash>.woff2`.
- **`web/LICENSES.md`.** Its row; a JetBrains Mono section, whose licence is the OFL text already quoted; and the GSAP note of §10.1.
- **`net/asset-fetch.ts`.** The types `LandingSlot` and `LandingClip`, and `LANDING_CLIPS`, empty for now (D-325: every slot shows its still). P156 fills it.
- **New in `ui/landing/`.**
  - `parts.tsx`: `MotionScope`, `Eyebrow`, `SectionHead`, `CtaLink`, `PhoneFrame`, `Chip`.
  - `LoopVideo.tsx`: §9.5 and D-326.
  - `gsap.ts`: `loadGsap()` and `useGsap()`, unused for now.
  - Sections: `TopBar`, `Hero`, `TryIt`, `Pricing` (today's content), `Updates`, `SiteFooter`.
- **New styles.** `landing.module.css` (root, tokens, top bar, hero, shared parts) and `landing-sections.module.css`.
- **`LandingPage.tsx`.**
  - The root is `<div data-landing data-theme="dark">`, holding the top bar, `<main>` with the sections, and the footer.
  - The props and the navigation effect are unchanged: `git diff 8d94a06` shows none of the effect's lines.
  - `capabilityLine()` moved verbatim to `Hero.tsx`.
- **`landing.spec.ts`.** Two new cases at the end, L-3 (for now: the top bar's and the hero's "Start free") and L-10. No existing case changed.

**What differs from the plan.**

- **The Motion scope is in `parts.tsx`.** §10.1 puts `<LazyMotion>`/`<MotionConfig>` in `LandingPage.tsx`, but D-319's lint rule refuses `motion` outside `ui/landing/`. So `LandingPage.tsx` renders `MotionScope` from `parts.tsx`.
- **`LANDING_CLIPS` and its types come now, empty.** `LoopVideo` and the sections need the types. The paths still come in P156.
- **The first design read** (captures at 1280, 1920 and 390, read on D1) found the page's margin reset outranking the section classes. The reset is now under `:where()`, which has no weight.
- **The DropZone and NotifyMeForm restyle uses V1's colour names.** These are the names their shared classes read: the landing root sets `--color-*` to the dark tokens, as `.signIn` already does in `components.module.css`. The selectors of §11 adjust sizes. Neither component's file changed.

**Checks.**

| Check | Result |
|---|---|
| `pnpm check` | pass. `check-file-tree`: 503 files; longest is `landing.module.css`, 655 lines |
| Vitest | 522 passed |
| `pnpm build` | `check-hosts` ok, no new literal; app shell, gzip, **387.0 kB** (main chunk 195.6 kB, CSS 10.6 kB). The 33 kB rise is mostly Motion's `domAnimation` and `LazyMotion` |
| `pnpm e2e --workers=4` | 46 of 47 passed: the 18 old landing cases, L-3 and L-10, signin and checkout. "Two tabs" failed, then passed alone (Entry 3's flake) |
| Tier 3 | `pnpm build:e2e` (keyed) then `rejections.spec.ts --grep REJECT_FILE_SIZE`: passed, so `/app`'s drop zone is untouched |
| Captures | No security-policy violation and no sideways scroll at 1280, 1920 and 390 |
| Frozen check | empty |

The first `build:e2e` failed in Cargo ("could not parse/generate dep info … os error 3"). The founder had deleted `target/` (84 GB) while it ran. The second, from a clean `target/`, passed.

**Open.** Nothing new.

**Found, not fixed.** None.

## Entry 6 - 2026-11-05: P153, every feature, every "Coming soon", the way in

**What changed.**

- **New sections in `ui/landing/`**, each with its `id` and timecode eyebrow:
  - `Marquee`;
  - `BeforeAfter`;
  - `Features`: the 8 cards of §6, each with its CSS illustration. The `captionStyles` and `visuals` slots go through `LoopVideo`, with the illustration as children;
  - `HowItWorks`;
  - `Privacy`: its three facts and a "See the full list" link. Neither the privacy line nor a second "What leaves your device" link (D-334);
  - `ComingSoon`: "Next" and "On the way", every card with the chip, then the footnote;
  - `FinalCall`.
- **`parts.tsx`.** `BentoCard`.
- **`Pricing.tsx`** (D-329, option (a)):
  - the Free card has "Every feature above" and a "Start free" link to `appPath`;
  - the Creator card has the tag "For regular posting" and the chip "On sale soon", with no button and no link;
  - the section's lead is `notOnSale`.
- **`LandingPage.tsx`.** It composes the 12 sections in §6's order.
- **Styles.**
  - `landing-sections.module.css` grows to 574 lines.
  - **New: `landing-cards.module.css`** (436 lines): the bento cards and the "Coming soon" cards. It is split off to keep each file under 700 lines (§10.1 allows a split). It has its TS §5 row.

**DECLARED CHANGE (§12.4).** In `landing.spec.ts`, the case "pricing: the plans show their prices and limits, say they are not on sale, and offer no way to buy":

- the title gains ", only a way in";
- `pricing.getByRole("link")` goes from a count of 0 to 1;
- a new line checks that this link is named `cta.startFree` with `href` `/app`.

`pricing.getByRole("button")` stays at 0, and every price and feature line is unchanged. No other existing case changed.

**New cases.** L-1 (each card's title; no sideways scroll at 1280, 1920 and 390), L-2 (12 soon cards, each with the chip, its title and its line; the footnote) and L-4 (none of the waitlist's sentences on the page). L-3 now expects 4 "Start free" links.

**§4, row by row: where each feature is named on `/`.**

| # | Where |
|---|---|
| F1 | Try it: the limits line (`LIMITS`), and the drop zone's own prompt |
| F2 | Try it: the drop zone's "Try a sample" button |
| F3 | Card "Transcribed on your computer"; before-after step |
| F4 | Card "A cleaner voice, same length"; marquee; before-after step |
| F5 | Card "Captions that follow your voice" (Clean, Bold, Mono, accent numbers) |
| F6 | Card "Visuals for what you say" (Big number, List, Before → after, Emphasis; "turn any of them off") |
| F7 | Card "Vertical, framed your way" (pass through, crop, you pick it) |
| F8 | The same card (top, middle, bottom, drag) |
| F9 | Card "Fix any word in a click" (live preview with sound) |
| F10 | The same card (correct a word, change every copy, hide it) |
| F11 | The same card (undo, redo, keyboard shortcuts) |
| F12 | Not marketed (§4) |
| F13 | Card "Ready to post"; pricing |
| F14 | Card "Ready to post" ("a short summary of what we changed") |
| F15 | Card "Sign in with a link" |
| F16 | Pricing |
| F17 | Card "Sign in with a link" (your plan, your data, delete your account) |
| F18 | Hero facts (privacy line and link); the privacy section |
| F19 | Hero facts; the capability line |

All 12 rows of §5 are cards with "Coming soon" (case L-2).

**What differs from the plan.**

- "Every feature above" is a list item of its own on the Free card. It is not part of `pricing.free.features()`, which the pricing case loops over.
- Creator's "goes on sale soon" sentence is the section's lead (`notOnSale`), so the card carries a short chip, "On sale soon", rather than the sentence a second time.

**The design read.** Captures at 1280 and 390, read on D1, led to three fixes:

- the last "Coming soon" card takes a whole row in the three-column grid, where it stood alone;
- "1080p" and "720p" keep their lower-case "p";
- the captions card's phone is larger.

**Checks.**

| Check | Result |
|---|---|
| `pnpm check` | pass. 511 files in the tree |
| Vitest | 522 passed |
| `pnpm build` | `check-hosts` ok; app shell, gzip, **391.6 kB** (main chunk 198.1 kB, CSS 12.7 kB) |
| `pnpm e2e --workers=4` | 49 of 50 passed: the 17 unchanged landing cases, the declared pricing case, L-1 to L-4 and L-10, signin and checkout. Checkout's "Free: the plan and its count…" failed (`toHaveURL` of the merchant's checkout within 5 s). Alone, `--repeat-each=3`: 3 passed. It is one of Entry 3's load-flaky cases, on the account page, which Part A does not touch |
| Frozen check | empty |

**Open.** Nothing new.

**Found, not fixed.** None.

## Entry 7 - 2026-11-05: P154, Motion

**What changed.**

- **M7, the section entrances.** `useRise()` in `parts.tsx` gives `initial { y: 16 }`, `whileInView { y: 0 }`, `once`, `amount: 0.3`, 0.6 s, the brand ease, and 0.06 s between siblings. It is used by:
  - every `SectionHead`;
  - the before-after steps;
  - the 8 bento cards;
  - the privacy facts;
  - the two plans;
  - the 12 soon cards (staggered by row).

  Content only moves, it never fades (rule 2.5). Under reduced motion the hook gives no animation props at all, so nothing is registered: this does not rely on `MotionConfig` alone.
- **M5, the hero chips.** They enter one after another (0.12 s apart) from opacity 0 (decoration) and 12 px down. Then they drift ±6 px for ever (6-8 s, `easeInOut`). Under reduced motion they are plain spans.
- **M8, the bento spotlight.** A pointer handler writes `--x` and `--y` with `style.setProperty` on the card, and a radial gradient follows them. The hover lift uses `translate`, so it does not fight Motion's `transform`.
- **The top bar's hairline.** It appears after 8 px of scroll, through Motion's `useScroll` and `useMotionValueEvent`, which toggle a `data-scrolled` attribute (no style is written).
- **M11, in CSS,** every keyframe under `(prefers-reduced-motion: no-preference)`:
  - the waveform, noisy then clean;
  - the crop window sliding;
  - the editor card's word swap;
  - the transcript's words appearing;
  - the "Coming soon" chips' dot pulse;
  - Creator's violet sweep along its border (`@property --sweep` and a masked conic gradient).

  Still, each picture shows its final state: the clean waveform and the fixed word.
- **New cases.**
  - **L-5:** under `emulateMedia({ reducedMotion: "reduce" })`, after a scroll through and 1 s, `document.getAnimations()` is empty, the headings are there, and there is no pin spacer. The clip and pill checks join when they exist (P155, P156).
  - **L-7:** a scroll from top to bottom in 20 steps records no security-policy violation, and `crossOriginIsolated` stays true.

`git grep -n "opacity: 0" web/src/ui/landing` has two hits, both decoration:

- `Hero.tsx`, the floating chips' entrance (M5);
- `Privacy.tsx`, the facts' check icons (§6 says only the icons fade).

**No** `layout`, `layoutId`, `AnimatePresence` or `style` prop is used.

**What differs from the plan.** None.

**Checks.**

| Check | Result |
|---|---|
| `pnpm check` | pass. Longest file `landing.module.css`, 663 lines |
| Vitest | 522 passed |
| `pnpm build` | `check-hosts` ok; app shell, gzip, **397.6 kB** (main chunk 203.7 kB, CSS 13.2 kB) |
| `pnpm e2e --workers=4` | **52 of 52** passed, including L-5, L-7 and the two `page.clock` cases (`landing_view`, `capability_check`) |
| Frozen check | empty |

**Open.** Nothing new.

**Found, not fixed.** None.

## Entry 8 - 2026-11-05: P155, GSAP

**What changed.** Each animation runs through `useGsap()`: after `loadGsap()`, inside `gsap.matchMedia()` under `(prefers-reduced-motion: no-preference)`, and reverted on unmount.

| # | Where | What |
|---|---|---|
| M1 | `CaptionedHeadline.tsx` (new) | The H1's words are spans with real spaces, so its accessible name is unchanged. A yellow pill (`aria-hidden`) slides from word to word, 0.28 s each, `power3.out`, lights each word in turn (`data-lit`), and rests on "finished.". It is measured with `getBoundingClientRect`, and measured again on resize |
| M2 | `TopBar.tsx` | The 2 px bar's `scaleX` is scrubbed over the whole page (`scrub: 0.3`) |
| M3 | `TopBar.tsx`, `Playhead` | At 1280 px and wider, a dot travels a line down the left gutter. Each section's eyebrow gets `isLive` between `top center` and `bottom center`. `Playhead` is a sibling of the bar, not inside it: the bar's `backdrop-filter` would make it the box a fixed child is placed in |
| M4 | `Hero.tsx` | The phone turns towards a fine pointer, up to 6° (`quickTo`). It is off for touch (`(pointer: fine)`) |
| M6 | `Marquee.tsx` | `xPercent: -50`, repeating, 40 s a loop. It slows to 0.3 on hover, and moves only while on screen |
| M9 | `HowItWorks.tsx` | At 1024 px and wider the section is pinned (`end: "+=150%"`), and the scroll moves the active step on. The others stay readable at opacity 0.5 (`data-pinned`, `data-active`) |
| M10 | `FinalCall.tsx` | M1's pill runs once over the final line and rests on "away.", from `top 60%` |

**What differs from the plan.**

- **One runtime module, `gsap-runtime.ts` (new, with its TS §5 row).** It imports `gsap` and `gsap/ScrollTrigger` statically, and `gsap.ts` imports it with `import()`. With two separate `import()`s, the main bundle would have held the chunk name `ScrollTrigger-<hash>.js`, and a `.ScrollTrigger` property name survives minification. Either would put the text "ScrollTrigger" in `/app`'s scripts and fail L-8, though GSAP never loads there. The tools object therefore calls the plugin `trigger`.
- **No GSAP download under reduced motion.** `useGsap()` returns before `loadGsap()` when `(prefers-reduced-motion: no-preference)` does not match: nothing would run anyway. Without this, ScrollTrigger kept its own 60 fps keep-alive loop running with no trigger registered.
- **`check-hosts.mjs`** (§10.1 allows this change) tolerates one literal, `https://gsap.com`, exactly, in `assets/gsap-runtime-*.js` only. It is the address in the text of GSAP's console warning for a missing target, and the library makes no request. "Every request of a session goes to the page's own origin or to the asset host" still passes.
- **GSAP's banner.** The production build minifies GSAP and keeps none of its `@license` banner comments. Entry 4 and `web/LICENSES.md` assumed it would: both are corrected. Keeping the banners needs a `vite.config.ts` option, outside §10.1's files. It is listed under Open items for the founder.

**The page under `page.clock` (found here, kept).**

- `landing_view` and `capability_check` install a fake clock, then advance it 10 s. That replays about 600 animation frames in a row.
- ScrollTrigger keeps a trivial `requestAnimationFrame` loop alive while it is in use. GSAP's own comment says this is for consistent repaints.
- Replaying it makes the two cases slower: alone about 10 s, where they took 3-4 s at P154; 12-13 s in the full run at 4 workers. They pass, and their code is unchanged.
- In a normal browser the same page costs **32 ms of script and 0 ms of layout in 3 s of idle** (CDP `Performance.getMetrics`). So visitors are not affected.
- With 6 workers on D1 the load made three cases fail once (L-5 among them). They passed with 4 workers and alone. The gate's 4 workers are unchanged.
- CI runs fewer workers, with one retry. **For the merge:** CI's timings for these two cases are read before merging.

**The long-task read (guide Phase 5).** On a production build in Chrome on D1 at 1280 × 800, scrolling the whole page by wheel with a `longtask` observer recorded **no task over 50 ms**, with movement on and under reduced motion.

**New cases.** L-8 (the scripts of `/app` hold no "ScrollTrigger"; after a scroll of `/`, one does) and L-9 (after "Start free" in the top bar: no `.pin-spacer`, no `[data-landing]`, no page error). L-5 also checks that every pill has opacity 0 and that no word is `data-lit`.

**Checks.**

| Check | Result |
|---|---|
| `pnpm check` | pass. 513 files in the tree |
| Vitest | 522 passed |
| `pnpm build` | `check-hosts` ok (one new tolerated literal, above); app shell, gzip, **443.4 kB** (main chunk 205.3 kB, **GSAP chunk 44.1 kB**, CSS 13.2 kB), under the 500 kB of D-320 |
| `pnpm e2e --workers=4` | 53 of 54 passed. "Two tabs" failed and passed alone (Entry 3). J1 and both `page.clock` cases passed unchanged |
| Captures | The pill rests on "finished." and on "away."; the hero's timecode is lit by the playhead |
| Frozen check | empty |

**Open.** Item 5 (new): GSAP's banner comments in the bundle.

**Found, not fixed.** None.

## Entry 9 - 2026-11-05: P156, the clips

**Step 6.1, the re-encode.** The seven web files were encoded from their masters in `offcut-remotion/out/` into `~/design-upload/`, with guide Step 6.1's command. The `out/` folder was only read. The §9.2 probe of the results:

| Clip | Size | Codec | Pixels | fps | Frames | Audio | Faststart | Bytes (cap) |
|---|---|---|---|---|---|---|---|---|
| hero | 720 × 1280 | H.264 High | `yuv420p`, tv, BT.709 | 30 | 300 | none | yes | 599,441 (2,500,000) |
| before-after | 1280 × 720 | H.264 High | `yuv420p`, tv, BT.709 | 30 | 360 | none | yes | 393,358 (3,000,000) |
| caption-styles | 720 × 1280 | H.264 High | `yuv420p`, tv, BT.709 | 30 | 270 | none | yes | 298,252 (2,000,000) |
| visuals | 960 × 960 | H.264 High | `yuv420p`, tv, BT.709 | 30 | 360 | none | yes | 293,808 (2,500,000) |
| how-it-works | 1280 × 720 | H.264 High | `yuv420p`, tv, BT.709 | 30 | 360 | none | yes | 723,654 (2,500,000) |
| privacy | 1280 × 720 | H.264 High | `yuv420p`, tv, BT.709 | 30 | 180 | none | yes | 44,173 (1,500,000) |
| coming-soon | 1280 × 720 | H.264 High | `yuv420p`, tv, BT.709 | 30 | 240 | none | yes | 214,432 (1,500,000) |

Every frame count is unchanged. A middle frame of each was read on D1 beside the same frame of the original: the colours match.

**Step 6.3, the upload.** The founder's go is given by `overnight-run.md` §3.2. `grep -c '^ASSET_S3_' .env.deploy` printed 4. The seven files were uploaded once, with `sh scripts/upload-assets.sh media …` in a subshell. No key was printed, and nothing else on the asset host was touched. The printed paths:

- `media/offcut-hero-720x1280.638a910246cea94f.mp4`
- `media/offcut-before-after-1280x720.8302b0f909c47763.mp4`
- `media/offcut-caption-styles-720x1280.d2b8c6103512e7d2.mp4`
- `media/offcut-visuals-960x960.7d7b21acf8b32466.mp4`
- `media/offcut-how-it-works-1280x720.bf1c3a7343abb4b6.mp4`
- `media/offcut-privacy-1280x720.642f1e2cda617aaa.mp4`
- `media/offcut-coming-soon-1280x720.5fff4a66a864300e.mp4`

`curl -sI` with `Origin: https://offcut-one.vercel.app` on each answered:

- `200`;
- `video/mp4`;
- `Cache-Control: public, max-age=31536000, immutable`;
- `Access-Control-Allow-Origin: https://offcut-one.vercel.app`.

`http://localhost:4173` is allowed too, so L-6 runs locally. The hand-made demo stays on the asset host; nothing references it.

**Code.**

- **`net/asset-fetch.ts`.** `LANDING_CLIPS` holds the seven paths and sizes. `DEMO_VIDEO_PATH` is now the hero clip's path (D-324), through a `HERO_CLIP` constant. `git grep -n "demo.81cafa494d9c2b68" web/src` is empty.
- **Posters.** The seven `.webp` posters are in `web/src/ui/assets/landing/<slot>.webp` (5.7-11.9 kB each, over Vite's 4 kB inline limit, so emitted hashed under `/assets/`). Each section passes its poster to `LoopVideo`.
- **`web/public/og.png`** (1200 × 630, 143 kB): the headline in Inter 800 with the yellow pill, beside a frame of the hero clip, on `#0e0e11`.
- **`web/index.html`.** A title, a description, `og:type`, `og:title`, `og:description`, `og:image` (`https://offcut-one.vercel.app/og.png`), its width and height, `og:url` and `twitter:card`. No script and no other host.
- **`check-hosts.mjs`.** It tolerates `https://offcut-one.vercel.app/`, in `index.html` only, with its reason: the site's own address in the open-graph tags, which a link preview reads.
- **New case L-6.** For each of the 7 clips, scrolled into view: `readyState >= 2`, no error, playing, and `currentSrc` on the asset host; `crossOriginIsolated` stays true. L-5 now also checks that, under reduced motion, every clip is paused on its poster with a "Play the clip" button.

**What differs from the plan.**

- **The hero clip and the caption-styles clip are not framed in a CSS phone.**
  - Both clips draw their own phone. The hero also draws its own three chips ("Voice cleaned", "Captions, word by word", and the counted number): `remotion-prompts.md`, hero, 3.0-9.0 s, and "the page will place it to the right of the headline".
  - In the page's phone, the read of the captures showed a phone inside a phone, and two sets of the same chips.
  - So with a clip, the page places it in a rounded 9:16 box (`.heroClip`, `.tallClip`).
  - The CSS phone and M5's floating chips stay as the still for a missing clip (D-325). With all seven clips uploaded, M5 is not shown. M4's tilt applies to the clip's box.
- **`og.png` uses a frame from the middle of the hero clip**, not its poster. The poster is the clip's first frame, nearly black ("Raw take").

**Checks.**

| Check | Result |
|---|---|
| `pnpm check` | pass. 521 files in the tree; longest `landing.module.css`, 675 lines |
| Vitest | 522 passed |
| `pnpm build` | `check-hosts` ok; app shell, gzip, **444.2 kB** (main 205.7 kB, GSAP 44.1 kB, CSS 13.2 kB, `index.html` 0.7 kB) |
| `pnpm e2e --workers=4` | 54 of 55 passed, including L-6 (10.9 s), `@smoke` and "every request of a session…". "Two tabs" failed and passed alone (Entry 3). The `page.clock` cases took 7.9 s and 8.5 s |
| The unsupported page | Opened at `/app` under Playwright's Pixel 7 emulation: `UNSUPPORTED_MOBILE`, and its video's `currentSrc` is the hero clip, `readyState` 4, 720 × 1280, no error |
| Frozen check | empty |

**Seen in the clips, for the founder.** The how-it-works clip shows the editor's real screenshots, with a person's face in the preview. Entry 2's read accepted this clip under the truth rules (§9.2: "the only person on screen is the founder"). This run takes that face to be the founder's own footage. If it is not, re-render the clip and upload it again (a new path; the slot shows its poster meanwhile).

**Open.** Item 2 is closed (the upload).

**Found, not fixed.** None.

## Entry 10 - 2026-11-05: P157, the other pages on the dark system (Part C)

**What changed.**

- **One attribute per page.** `data-theme="dark"` on the root `<main>` of six pages: `SettingsPage`, `LegalPage`, `AccountPage`, `AuthCallbackPage`, `UnsupportedPage`, and `CapabilityGate`'s checking state. Each `.tsx` diff is that attribute and nothing else.
- **`pages.module.css`, one block at its end.**
  - `.page[data-theme="dark"]` maps V1's colour names (`--color-bg`, `--color-surface`, `--color-text`, `--color-muted`, `--color-border`, `--color-accent`, `--color-on-accent`, `--color-ok`, `--color-problem`) to the dark tokens. Every rule above it, and the shared form and button classes of `components.module.css`, read these names, so no markup changes.
  - The pages keep their 16 px body (the dark tokens set the editor's 14 px).
  - The account page's deletion dialog gets a dark backdrop. Its `::backdrop` mixes `--color-text`, which would turn light on a dark page.
- **The font** is Inter UI, from the dark tokens.
- **New case L-12.** `axe(page)` on `/settings` and `/legal/terms`, each with its `main[data-theme='dark']` and its H1: no serious or critical violation. `landing.spec.ts`'s import line gains `axe` from `helpers/fixtures.ts`; no case changed.

**Frozen-check exceptions** (named by P157): the six `.tsx` and `pages.module.css`. Only `ui/components/CapabilityGate.tsx` is under a frozen path, and the check prints only it.

**Tier 3.**

- `signin.spec.ts` and `checkout.spec.ts` are non-media, so Tier 2 runs them in full.
- Of the media suites, only `export-free.spec.ts` reaches the account page. Its case "the top bar's Account link opens the account page in a tab of its own" is part of a serial group that first processes a clip end to end, so it cannot run alone.
- It runs in CI's `e2e-media` on the pull request, and in Part B's full media run.

**Checks.**

| Check | Result |
|---|---|
| `pnpm check` | pass |
| Vitest | 522 passed |
| `pnpm build` | `check-hosts` ok; app shell, gzip, **444.2 kB** |
| `pnpm e2e --workers=4` | 55 of 56 passed, including L-12 and all of signin and checkout. "Two tabs" (signin, on the account and callback pages this prompt touches) failed in the full run, then passed **3 of 3** alone with `--repeat-each=3`: Entry 3's flake |
| `git diff 8d94a06 -- web/tests-e2e/signin.spec.ts web/tests-e2e/checkout.spec.ts` | empty |
| Captures | `/settings`, `/legal/terms` and `/account` on the dark system, read on D1 at 1280 × 800 |
| Frozen check | `web/src/ui/components/CapabilityGate.tsx` only, the named exception |

**Open.** Nothing new.

**Found, not fixed.** None.

## Entry 11 - 2026-11-05: P158, axe, the captures, design review 3

**L-11.** The first run of `axe(page)` on `/` at 1280 × 800 found `color-contrast` (serious) on 4 nodes at the top, and 4 at the end. They were the step numbers and lines of the pinned "How it works" steps that the scroll had not reached (P155's `opacity: 0.5`: 2.94:1 and 2.77:1).

The fix: a resting step keeps text of full contrast, and only its title takes `--text-2`. The active step still stands out by its violet border and glow. After it, axe reports nothing serious or critical at the top or at the end, at 1280 × 800, 390 × 844 and 1920 × 1080. The case L-11 covers 1280 × 800.

**The captures.** `screens.spec.ts` has a new loop, "landing", over 1280 × 800, 1920 × 1080 and 390 × 844, with and without movement:

- a scroll through, then 3 s at the top;
- a full-page and a first-screen picture each.

They were made from the development build (`vite build --mode development`, as `pnpm screens` does), running only `--grep landing` (the screens project's other states need a clip end to end and are unchanged). The 12 pictures are in `~/design-review/`, none staged. Each full page was also cut into strips and read at full size.

**Design review 3, self-review** (`overnight-run.md` §5: the founder delegated it tonight). Two rounds.

| # | Rubric | Round 1 | Round 2 |
|---|---|---|---|
| 1 | The complaint answered; the first screen at 1280 × 800 has the headline, subhead, "Start free" and the hero clip; nothing looks like default browser styling | yes | yes |
| 2 | Every feature of §4 for the page named in plain words; every row of §5 with "Coming soon" | yes (Entry 6's table; L-1, L-2) | yes |
| 3 | No overlap, no cut-off words, no sideways scroll at 390 | yes ("talking-head" breaks at its own hyphen) | yes |
| 4 | One content width, even rhythm, grids that line up | yes | yes |
| 5 | Readable contrast; the yellow pill only in the H1, the final call and the Captions card | **no**: axe's 4 nodes (above), and the wordmark's "O" was a yellow pill (§6's draft for section 0) | yes: the pinned steps fixed; the wordmark's "O" is now violet (`--brand-hover`, white text) |
| 6 | No placeholder text, no invented number or testimonial, no other company's name or logo | yes. The only company named is "Dodo Payments", in the merchant line kept from V6 because the law asks for it. The clips' sample captions ("3,000 hours", "10,000 subscribers") are example content inside the product's own frames, read at G-1 | yes |
| 7 | Under reduced motion everything present and still | yes: no pill, no pin, every clip on its poster with "Play the clip" | yes |
| 8 | Drop zone, "Try a sample" and the email form look like what they are and work | yes; their cases pass unchanged | yes |

**Found in the review and fixed (round 1).** At 1920 × 1080 the hero clip stood on its poster with "Play the clip", although movement was allowed.

- The scroll through had paused it before its `play()` had started.
- The browser rejects that `play()` with an `AbortError`, and `LoopVideo` treated every rejection as a refusal, for good.
- Now only `NotAllowedError` counts as a refusal. In round 2 every clip played.

**The result: self-review, rubric passed** (round 2, 2026-11-05), in place of the founder's "go". The founder reads the same captures and the copy of `copy/landing.ts` in the morning, and can still ask for changes after the release is live.

**Checks.**

| Check | Result |
|---|---|
| `pnpm check` | pass. Longest file `landing.module.css`, 676 lines |
| Vitest | 522 passed |
| `pnpm build` | `check-hosts` ok; app shell, gzip, **444.3 kB** |
| `pnpm e2e --workers=4` | **57 of 57** passed, L-11 among them |
| Captures | 12 in `~/design-review/`, none staged |
| Frozen check | `CapabilityGate.tsx` only (P157's exception) |

**Open.** Item 3 (design review 3) is done as a self-review; the founder's own read is in the morning report.

**Found, not fixed.** None.

## Entry 12 - 2026-11-05: P160a, the copy-back (Parts A and C) and the gates before the merge

**The copy-back of §20, for Parts A and C.** DC-13 waits for Part B's own pull request.

| Spec | What changed |
|---|---|
| TS §5 | Done with the prompts (P151, P153, P155): every new file has its row, `gsap-runtime.ts` and `landing-cards.module.css` included |
| TS §13.1, the `/` row | Hero with the captioned headline and the hero clip; sections of features and "Coming soon"; `DropZone`; pricing with a way in; the update form; Motion and GSAP (TDR-26) |
| TS §13.5 | The app-shell budget is 500 kB (D-320) |
| TS §39.3 item 24 | 500 kB, with this release's reading: 444.3 kB, main 205.8 kB, GSAP chunk 44.1 kB; no task over 50 ms while scrolling |
| TS §38 | TDR-26: GSAP and Motion, on the landing page only |
| INV-15 | Done in P151: "`copy/messages.ts` and the files it imports" |
| PS §10 J1 | The new hero line, the clips in place of the hand-made demo, the sections |
| PS §11 | Unchanged prices. A note: "Start free" on Free, "On sale soon" on Creator until the founding release |
| BP | A paragraph for the design release, between V6 and the cloud-transcription release |

`git grep -n "TODO\|FIXME" web/src/ui/landing` is empty.

**The gates of `overnight-run.md` §7.2, before the first merge.**

- [x] **Local, on the head of this commit's code (`e3f2548`; this commit adds documents only):**
  - `pnpm check` passes;
  - `cargo test --workspace`: 812 / 0 / 4, as at Setup (Part A touches no Rust);
  - Vitest 522, as at Setup;
  - `pnpm build`: `check-hosts` ok, app shell 444.3 kB (≤ 500);
  - `pnpm e2e --workers=4`: **57 of 57**. Setup's floor was 45; the 12 new cases are L-1 to L-12.
- [x] **L-1 to L-12 pass** (Part C was built, so L-12 counts).
- [x] **The unchanged cases.** The 17 unchanged landing cases, `signin.spec.ts` and `checkout.spec.ts` pass with no change to their code:
  - `git diff 8d94a06 -- web/tests-e2e/signin.spec.ts web/tests-e2e/checkout.spec.ts` is empty;
  - in `landing.spec.ts`, the only lines removed are the pricing case's title and its `link` count of 0 (§12.4's declared change), and the import line that now also takes `axe`. Everything else is added.
- [x] **The targeted media cases on a keyed build.**
  - P152's `rejections.spec.ts --grep REJECT_FILE_SIZE` passed on `pnpm build:e2e`.
  - P157 has no case that can run alone (Entry 10). CI's `e2e-media` ran the media suites on the pull request instead (below).
- [x] **The frozen check** prints only `web/src/ui/components/CapabilityGate.tsx`, Part C's named file.
- [x] **The changed files.** `git diff --stat 8d94a06` (outside `docs/`) lists only these 50 files:
  - §10.1's files;
  - Part C's seven;
  - `gsap-runtime.ts` and `landing-cards.module.css`, recorded in Entries 6 and 8;
  - `screens.spec.ts`'s landing loop (§12.5);
  - the lock file.
- [ ] **CI on the pull request's head, every job, `e2e-media` included.** The push after P156 (run 41908517780) was green in every job: `changes`, `ci` in 12 min 46 s, `e2e-media` in 15 min 57 s. The push of this commit starts the run that is read before the merge, and its result is recorded in the closing entry.
- [x] **The self-review rubric** passed (Entry 11).

**Open.** Nothing new.

**Found, not fixed.** None.

## Entry 13 - 2026-11-05: Parts A and C merged, deployed, checked and tagged

**The last gate.** CI on the pull request's head `d8618b6` (run 41913861090) passed in every job:

- `changes`;
- `ci` on its one re-run, 13 min 12 s;
- `e2e-media`, 14 min 57 s.

The first attempt of `ci` stopped in "Golden frames, with their readings". Every golden had passed, then the test process died with SIGSEGV. `git diff 8d94a06 -- crates fixtures` is empty, so the crash was the runner's software GPU (lavapipe), not this code. One re-run is allowed (`overnight-run.md` §6), and it passed.

On CI, `landing_view` took 5.1 s and `capability_check` 4.8 s (Entry 8's watch), and `pnpm e2e` passed 57 of 57 with no flake.

**The merge.** `gh pr ready 13`, then `gh pr merge 13 --merge`: merge commit **`80d611e`**. The `main` run 41920213592 passed in every job:

- `ci`, 13 min 42 s;
- `e2e-media`, 14 min 50 s;
- `deploy-api`, 2 min 18 s;
- `deploy-web`, 1 min 18 s: the build for Vercel, the test-key guard, the deploy, the API through the rewrite, the headers, and the smoke case against the deployment, every step green.

**The live checks of `overnight-run.md` §7.3** (2026-11-05, about 04:00 IST, from D1):

- `E2E_BASE_URL=https://offcut-one.vercel.app pnpm --filter web exec playwright test --project=non-media --grep @smoke`: 1 passed.
- `node scripts/check-headers.mjs https://offcut-one.vercel.app`: "sends the 7 headers of web/vercel.json, value for value". The policy and both isolation headers are present.
- `https://offcut-api.onrender.com/api/v1/healthz`: `{"ok":true,"version":"80d611e…"}`.
- `/` in Chrome, 1280 × 800, analytics answered locally so that no row is added:
  - the H1 reads "Say it once. Post it finished.";
  - all 7 clips reach `readyState` 4 with no error;
  - `crossOriginIsolated` is true;
  - no security-policy violation;
  - the top bar's "Start free" leads to `/app` inside the app, which shows the drop zone with "Try a sample" enabled;
  - no `.pin-spacer` and no `[data-landing]` are left behind.
- One console line was seen, a `401` on `POST /api/v1/auth/refresh`. That is the session check of a visitor who is not signed in, which V6 treats as normal ("a refusal is not an error"). It is not from this release.
- The screenshots are `~/design-review/live/live-top.png`, `live-end.png` and `live-app.png`, none staged.
- The editor is unharmed: on the live `/app` the drop zone shows and "Try a sample" is enabled. Nothing was exported, and nobody signed in.

**The tag.** `git tag -a design -m "Design release: landing page, clips" 80d611e`, pushed. So open item 4's tag part is done; the Dodo resubmission stays the founder's.

**Next.** Part B, lower captions (P159), on its own branch `design-captions` from `origin/main`. This entry is that branch's first commit.

## Entry 14 - 2026-11-05: P159, lower Bottom captions (Part B, DC-13): built, not shipped

**Branch** `design-captions`, from `origin/main` at the merge `80d611e`. The pull request is left a **draft**, under `overnight-run.md` §6 and §7.4.4.

**What was built.**

- **The stills first** (guide Step 9.1). With option A in the working copy, `golden_frames` was run without updating. Each changed golden wrote its new picture to `target/golden-out/`, beside today's in `fixtures/golden/`.
  - The 14 caption goldens (Clean, Bold and Mono, each kind, at Bottom, the default placement) and the Free frame with its watermark were read side by side. They are in `~/design-review/captions/`: one `<golden>-today-vs-A.png` each, and three contact sheets.
  - A moves a one-line caption about 90 lp lower and a two-line one about 45 lp lower. The Mono panel stays inside the block.
  - On the Free frame the pill stays above the "Made with Offcut" watermark, with nothing on it. Nothing leaves the safe area, and nothing reads worse than today. **A held.**
- **`captions.rs`.** The lines' top is now `block.y + (block.h − H)/2 + (align − 0.5)·(block.h − 2·inset − H)`. At 0.5 the second term is exactly `+0.0`, so a centred block is drawn bit for bit as before.
- **`placement.rs`.** `line_align(block)`, the block's place in the band (0 at `CAPTION_TOP`, 1 at `CAPTION_BOTTOM`), `pub(crate)`. `lines()` keeps its signature.
- **`styles.rs`** (outside P159's named exceptions, recorded here). `StyleSpec::decoration_height()`, purely additive: half the stroke with the drop shadow's 4 lp, the pill's padding down (4 lp), the panel's padding. That gives Clean 6.5, Bold 9, Mono 32.
  - It counts the shadow and the pill whether the style draws them or not. My first version read them from the style, and the existing unit test `captions_draw::the_shadow_and_the_pill_change_no_layout_box` failed: it holds V4's rule that the shadow and the pill change no layout box.
  - The test was not touched. The code now keeps the rule.
- **Four new unit tests** in `captions.rs`:
  - placement 0.5 is unchanged bit for bit, for Clean, Bold and Mono, one and two lines;
  - at 1.0 the last line with its decoration ends within 2 lp of the block's bottom, never below it;
  - at 0.0 the first line starts its decoration at the block's top;
  - each style's decoration height.
- **Snapshots.** `INSTA_UPDATE=always cargo test -p offcut-scene` rewrote 40 `.snap` files. A script compared every old file with its new one, field by field:
  - 206 glyph-run translations `transform.m[5]`;
  - 40 rect `y` values (pills and panels);
  - 2 layer bounds' `y`.
  
  Nothing else differs: no command was added, removed or reordered, and no horizontal value changed.
- **Goldens.** `OFFCUT_GOLDEN_UPDATE=1` rewrote 14 of the 16, and a second run without the variable passed 19 of 19. `crop_landscape_050`, `punch_in_110` and the parity case are unchanged.
- **TS §41.2.** The DC-13 row, with "founder's choice: option A, 2026-11-05".

**Tier 4, once, detached.**

| Check | Result |
|---|---|
| `cargo test --workspace` | **816 / 0 / 4**: Setup's 812 and the 4 new tests. `layout_safe_area.rs`, `hit.rs`, `parity.rs` and `placement.rs` pass unchanged |
| `pnpm check`, Vitest, `pnpm build` | pass; 522; shell 444.3 kB |
| `pnpm e2e --workers=4` | 52 of 57 in 3.8 min (1.6 min before): two checkout cases, L-11, L-12 and "two tabs" hit 30 s test timeouts or their 5 s waits. Run again alone on a fresh production build: **5 of 5 passed**. The web app has not changed since the deploy, which passed 57 of 57. D1 was under heavy load, after hours of builds |
| `pnpm build:e2e`, then the full `pnpm e2e:media` (55.6 min) | **98 passed, 2 failed, 1 skipped (`rej_h264_444`, as before), 6 did not run** |

**The two media failures.**

1. **`review-edit.spec.ts` 13, "drag the captions to the top: they snap at Top, and the move is one change": caused by DC-13.**
   - The protected helper `dragCaptions` (`tests-e2e/helpers/fixtures.ts`) takes the block "at its centre": it hovers at `blockCentre(position)`, y 1304 of 1920 at Bottom, and expects the hit test to answer `block` or `word`.
   - Under option A a one-line caption at Bottom sits about 90 lp below that point, so nothing is under the pointer: "no caption block under the pointer".
   - The 6 cases that did not run are the rest of that serial group.
   - Passing it would need either a change to the protected helper, or a larger hit region (the whole block), which is logic beyond §14's files. Neither is allowed tonight. So, under §6, **Part B is not shipped**.
2. **`timeline-preserved.spec.ts`, "Preview's length": not caused by DC-13.** It read 19,500 ms where it wants more than 19,500. Run again alone, the whole file passed, 11 of 11.

**For the founder: the choice for DC-13.**

- **(a) Recommended.** Declare a second line under DC-13: `dragCaptions` takes the captions where the player's mirror says the lines are, not at the block's centre. That is the helper's own premise ("the block is taken at its centre") meeting the change DC-13 makes. Then run `review-edit.spec.ts` (all of it) and merge.
- **(b)** Make the caption hit region the whole block. This is a behaviour change: a press 90 lp above the words would grab them.
- **(c)** Keep the captions as they are: close the pull request.

**Open.** Item 6 (new): Part B on the draft pull request, the choice above.

**Found, not fixed.** None in the code. The test helper's premise is above.

## Entry 15 - Morning report, 2026-11-05

**1. Live or not: live.**

- The new landing page and the dark pages are live on https://offcut-one.vercel.app from the merge **`80d611e`** (pull request 13). The tag is **`design`**.
- §7.3's checks all passed (Entry 13):
  - `@smoke` against the deployment;
  - the 7 headers equal to `web/vercel.json`;
  - the API at `80d611e`;
  - the H1 "Say it once. Post it finished.";
  - 7 of 7 clips at `readyState` 4;
  - isolated, with no security-policy violation;
  - "Start free" leads to `/app`, whose drop zone and "Try a sample" are as before.
- The live screenshots are `~/design-review/live/live-top.png`, `live-end.png` and `live-app.png`.

**2. What shipped and what did not.**

- **Part A shipped:** the landing page, Motion and GSAP, every working feature named, 12 "Coming soon", "Start free" four times, no waitlist sentence.
- **Part C shipped:** settings, legal, account, callback and unsupported on the dark system.
- **The seven clips shipped,** all uploaded and playing: hero, before-after, caption-styles, visuals, how-it-works, privacy, coming-soon.
- **Part B did not ship.** It is built, and its unit, snapshot and golden checks pass. But the protected `review-edit` case 13 fails under it, because its drag helper grabs the captions at the block's centre, where the lowered lines no longer are. It is on the draft pull request from `design-captions` (Entry 14, with the three choices).

**3. Every decision taken tonight** (`overnight-run.md` §3.1 and §6).

- GSAP's licence read as allowed: its one restriction on use is the Webflow-competitor one (Entry 4). The bundle drops GSAP's banner comments: open item 5.
- `MotionScope` lives in `parts.tsx`, because D-319's rule refuses Motion in `ui/pages/` (Entry 5).
- `LANDING_CLIPS` and its types came in P152, empty, and were filled in P156.
- `landing-cards.module.css` was split off to stay under 700 lines (Entry 6).
- Creator shows the chip "On sale soon". The sentence is the section's lead.
- `gsap-runtime.ts`: one chunk for GSAP and ScrollTrigger, so `/app`'s scripts never hold "ScrollTrigger" (L-8).
- No GSAP download under reduced motion. The marquee loops only while on screen.
- `check-hosts.mjs` tolerates two literals, each with its reason: GSAP's warning address in its chunk, and the site's own address in `index.html`'s open-graph tags.
- The hero and caption-styles clips are shown in rounded boxes, not CSS phones, because they draw their own phone and chips. M5's chips remain only for a missing clip.
- `og.png` is made from a middle frame of the hero clip.
- The upload: once, seven files, nothing else touched on the asset host.
- The pinned steps keep full contrast (axe), and the wordmark's "O" is violet (rubric item 5).
- `LoopVideo`: only `NotAllowedError` counts as a refused play.
- One CI re-run: the lavapipe SIGSEGV in "Golden frames", with no Rust changed.
- The merge, the deploy checks and the tag.
- Part B is left a draft under §6.

**4. Counts.**

| | Setup | Live (Parts A, C) | Part B branch |
|---|---|---|---|
| Rust | 812 / 0 / 4 | 812 / 0 / 4 | 816 / 0 / 4 |
| Vitest | 522 | 522 | 522 |
| `pnpm e2e` | 45 | 57 (CI 57, no flake) | 57 (5 load timeouts, each passed alone) |
| Media | 106 passed, 1 skipped (V6) | CI `e2e-media` green on the pull request and on `main` | 98 passed, 2 failed, 1 skipped, 6 did not run |
| App shell, gzip | 351.6 kB | 444.3 kB (GSAP chunk 44.1 kB; budget 500) | 444.3 kB |

**5. For the founder.**

- **The Dodo resubmission** (V6 CL open item 10). The live page no longer reads as a waitlist: "Start free" leads into the app, and pricing shows both plans with "Creator goes on sale soon".
- **Open item 17 (F-3):** hide the in-app offers until live mode, as its own change.
- **The clips** show a drawn silhouette, and the how-it-works clip shows a face in the editor's screenshots, taken here to be yours (Entry 9). Re-render with real footage if wanted: a new upload is a new path in `LANDING_CLIPS`, and the poster shows meanwhile.
- **Design review 3** was a self-review with the rubric (Entry 11). Read `~/design-review/` and `web/src/copy/landing.ts`. Any change is an ordinary pull request.
- **Part B:** choose (a), (b) or (c) of Entry 14. (a) is recommended.
- **Open item 5:** keep GSAP's banners in the bundle, or not.
- **`target/` grows large** (84 GB before tonight). After this release, `cargo clean` is safe; the next build starts slow.
- **The repository is public** (memory: for free CI). Nothing secret was committed tonight; `gitleaks` ran on every push and passed.

**6. Found, not fixed.**

- D1's `pnpm e2e` is load-sensitive. "Two tabs" and three checkout cases fail under parallel load and pass alone (Entry 3). At the end of the night, after hours of builds, D1 ran the suite at less than half its speed.
- Under `page.clock`, ScrollTrigger's keep-alive frame loop makes `landing_view` and `capability_check` slower to replay: 8-13 s on D1, 5 s on CI. A real browser is not affected (Entry 8).
- On CI, the golden test process crashed once with SIGSEGV after all its goldens had passed (lavapipe). A re-run passed.

## Entry 16 - 2026-11-05: Part B finished: "a little bit low", the positioning review, the helper (DC-13)

**The founder's answer to Entry 14.** Choice (a), with two conditions: "by lower I don't mean very low, little bit low from where they are today, because on real platforms have icons also at the bottom", and "do the review of the positioning first".

**The positioning review** (`~/design-review/captions-v2/`).

- **The method.** The 16 goldens at four settings: today (`6ea40bf`), 0.33 and 0.5 of A's movement, and A (Entry 14), in `today/`, `third/`, `half/` and `A/`. Six `compare-*.png` sheets put the four side by side, each with these marked:
  - the safe area's bottom (1500) and right edge (960);
  - the watermark's band (1444-1500);
  - three approximate regions where the apps a short is posted to draw their own interface: the right-hand button column (about x 940-1060, y 880-1600), the name and one-line description (about y 1540-1800), and a longer description growing up to about y 1450. These are boxes drawn by eye from common 1080 × 1920 layouts, not measurements of any one app.
- **The numbers** (glyph rows in lp, from the layout code itself):

  | At Bottom | Today | 0.5 | A |
  |---|---|---|---|
  | Clean, one line | 1260-1348 | 1305-1393 (+45) | 1350-1437 (+90) |
  | Bold, one line | 1253-1355 | 1293-1395 (+40) | +80 |
  | Mono, one line (panel to) | 1267-1341 (1373) | 1303-1377 (1409, +36) | +71 |
  | Clean, two lines | 1217-1391 | 1240-1414 (+23) | +46 |
  | Bold, two lines | 1202-1406 | 1217-1420 (+15) | +29 |

- **What it shows.**
  - Under A a one-line Clean caption rests on the watermark's band, and Mono's panel reaches the region where a longer description sits.
  - At 0.5, every style's last line ends in one narrow row, 1377-1420, 30-70 lp above where a longer description starts and 80-120 lp above the safe area's bottom. A one-line caption is centred at about 70% of the height (today 68%, A 73%).
  - 0.33 is hard to tell from today on a phone.
  - Nothing at any setting moves sideways, so the button column is as clear as today (the safe area's 120 lp right inset).
- **Chosen: 0.5.** It is "a little bit low" and keeps clear of the apps' bottom regions.

**What changed from Entry 14's code.**

- **`placement.rs`.** `line_align` is replaced by `line_drop(block)` and the constant `BOTTOM_LINE_DROP = 0.5`. The share of the room under centred lines that they move down is 0 at or above the band's middle, rising evenly to 0.5 at Bottom.
- **Only placements below the middle move.** A also moved Top captions up, nearer the apps' top bar, which nobody asked for. Every frame at placements 0 to 0.5 is now drawn as before, bit for bit. The Middle snap (0.55) moves about 4 lp.
- **`captions.rs`.** `top = block.y + centred + line_drop(block) · max(centred − decoration_height, 0)`, where `centred = (block.h − text)/2`. The `max` keeps a block too full to have room from moving up.
- **`styles.rs`.** `decoration_height()` is unchanged from Entry 14.
- **The four unit tests.** They are rewritten for the rule, and the decoration-height test is kept:
  - placements 0, 0.15, 0.3 and 0.5 are drawn bit for bit as centred, for all three styles, one and two lines;
  - at Bottom the lines move exactly 0.5 of their room and leave at least as much under them in the block;
  - a one-line Clean caption at Bottom moves 40-50 lp and keeps 40 lp or more under it;
  - between the middle and Bottom the drop rises evenly: at 0.75 it is half of Bottom's, and the Middle snap moves less than an eighth of it.
- **Snapshots.** `INSTA_UPDATE=always` rewrote the same 40 files as A. A script compared each with today's (`6ea40bf`), value by value:
  - 206 glyph-run translations `transform.m[5]`;
  - 40 rect `y` values (pills and panels);
  - 2 layer clips' `y`.

  There was no structural difference, and every change is between +14.7 and +45.0 lp.
- **Goldens.** `OFFCUT_GOLDEN_UPDATE=1` rewrote 14 of the 16 (as with A). A second run without it passed 19 of 19, with `crop_landscape_050`, `punch_in_110` and the parity case unchanged.

**DECLARED CHANGE (DC-13, second line): the protected e2e helper `dragCaptions`** (`web/tests-e2e/helpers/fixtures.ts`). It hovers at the block's centre and waited for `data-hit` to be `"block"` or `"word"`. The page never reports `"block"`: `PreviewGestures.tsx` writes the hit's kind, which is `"captions"` for the block (`HitValue::Captions`, since V4 P113). So the helper only ever matched a word near the block's centre, and that stopped being true once the lines were not centred. It now waits for `"captions"` or `"word"`, which is its own premise ("the block is taken at its centre"). A press on `"captions"` starts the same drag as a press on a word (`PreviewGestures.tsx`, `onPointerMove`). No other line of the helper or of any spec changed.

**TS.** The DC-13 row of §41.2 is rewritten for the half drop, the below-the-middle rule and the helper line, and adds `pipeline-preview.spec.ts` to its proofs. §19.1's caption-track paragraph gains one sentence. `designimplementation.md` §14 gains "As built".

**Checks.**

| Check | Result |
|---|---|
| `pnpm check` | pass (fmt, clippy, deny, eslint, both `tsc`, file tree, gen clean, copy codes) |
| `pnpm test` | Rust **817 / 0 / 4** (Setup's 812 and 5 new unit tests: 4 placement tests and the decoration-height test); Vitest **522** |
| `pnpm build:e2e`, then `review-edit.spec.ts` (all) and `preview-sync.spec.ts` | **16 of 16** and **4 of 4**. Case 13 (drag to Top) passed in 8.4 s, case 10 (click a caption word on the preview) in 6.2 s |
| `pipeline-preview.spec.ts`, "long tasks" (its caption drag) | passed; the drag had 0 tasks of 50 ms or more |
| `export-free.spec.ts`, `export-creator.spec.ts` | **32 of 32**, every file passing `verify_mp4.py`'s 11 checks: Free at 720 × 1280 with the watermark, and Creator. A first run failed 3 cases with `E_ENTITLEMENT_INVALID` because I had built without `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` (my mistake, not the change); after a keyed `pnpm build:e2e`, all passed |

No full media run. The founder's choice (a) is `review-edit` alone; the other DC-13 proofs were added because the frames changed from those of last night's full run. What DC-13 cannot reach (the other media specs) passed under A in Entry 14's full run, and CI's `e2e-media` runs the `@no-gpu` cases.

**Open.** Item 6 is closed by this entry and the merge.
