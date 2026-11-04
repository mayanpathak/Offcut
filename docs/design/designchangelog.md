# designchangelog.md - the design release, change by change

This file covers the design release between V6 and the cloud-transcription release. Its plan is `designimplementation.md`. Every change of the release is summarised here, in the same commit as the change.

## Open items

| # | Item | Who | Where |
|---|---|---|---|
| 1 | F-3: hide the in-app offers until live mode, as a small change of its own after this release (V6 CL open item 17) | Founder | §18 |
| 2 | The upload of the seven re-encoded clips needs the founder's go (P156) | Founder | guide Step 6.3 |
| 3 | Design review 3 (P158) | Founder | guide Phase 8 |
| 4 | The Dodo resubmission and the tag `design` (P160) | Founder | guide Phase 10 |
| 5 | GSAP's `@license` banners are not kept in the minified bundle. The notice is in `web/LICENSES.md` and the package. Keeping it in the bundle needs a `vite.config.ts` option (outside this release's files). Decide whether that is needed | Founder | Entry 8 |

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
