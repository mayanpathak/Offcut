# Offcut design release: coding-promptsdesign.md (Prompts 151-160)

**Source documents.**

- `docs/design/designimplementation.md`. Its sections are written **§n** and its decisions **D-n** (D-316 to D-335).
- `docs/design/designbuildguide.md`. Its steps are written **Step n.m**, and its list "Read this first" **bite n**.
- `TS`, `PS` and `BP` are the v1.3 specs.

These prompts were written after reading the code at `8d94a06` on 2026-11-05, with the founder's answers of §18 (every recommendation taken).

**How to use.**

- Paste the **Standard Agent Block (SAB)**, then one prompt, into your coding agent.
- Run them in order: Setup, then 151 to 160. Do not start a prompt until the one before it has its commit and every acceptance box ticked.
- Commits are titled `Design P<nnn>: <title>`.
- Steps marked **Human** are the founder's.
- Each prompt's **Effort** line is the reasoning effort to set.

**Design skill.** In P152 to P158, if the agent has a frontend or design skill, it loads it before writing interface code. This plan's tokens, rules 2.4 and 2.5, the copy and §12.1's unique strings win over any default the skill suggests.

---

# Standard Agent Block (SAB): prepend to every prompt

**Every terminal (Git Bash, never PowerShell or WSL)**

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
BASE=8d94a06
git branch --show-current     # design-build, always
```

**Before modifying anything**

1. Read the plan sections and guide steps this prompt cites, every file it touches, and the previous prompt's entry in `docs/design/designchangelog.md`.
2. Search before creating (`git grep -n "<name>"`).
3. Verify that the previous prompts' outputs behave as this prompt assumes. If not, make the smallest compatible correction and report it.
4. **Precedence.** D-316 to D-335 are binding for this release. Otherwise TS wins over the plan, and PS over both. If the sources disagree in a way that these do not settle, **stop and list it**.
5. **The founder's rule: nothing that works today may break, and no logic changes.** Before this prompt's first edit, run its Tier (guide Step 0.3) once and keep the result. After the last edit, run it again: every case that passed before still passes, unchanged.

**During implementation**

1. Implement only this prompt's scope.
2. **The frozen check** (guide Step 0.2) stays empty, except for the exceptions this prompt names.
3. **Tests.**
   - No existing test is edited, weakened, skipped or deleted, except the one pricing case of §12.4, in P153.
   - If any other existing case fails, the code is wrong, not the case: fix the code. If the case cannot pass without changing it, **stop and ask, with a recommendation** (TS §41.4).
4. **Files.**
   - A new file under `web/`, `scripts/`, `crates/` or `.github/` needs its TS §5 row in the same commit (bite 9).
   - No source file over 700 lines above its test module.
   - A temporary file is named `zz-…` or `zz_…` and is deleted before the gate.
5. **TypeScript.**
   - No `any`, `!`, `@ts-ignore` or empty `catch`. No disabled lint rule.
   - **No `style=` in JSX.** Values go through refs and `style.setProperty` in effects, or Motion's animation props (bite 7).
   - No `setAttribute("style", …)` and no `<style>` elements.
6. **Copy.**
   - Every word a person sees comes from `copy/` (`react/jsx-no-literals`).
   - No digit is typed into a sentence (bite 11).
   - The strings of §12.1 appear exactly once on `/` (bite 1).
7. **Motion.**
   - Content is never hidden by an initial state (rule 2.5). Decoration may fade; content may only move.
   - Reduced motion leaves a complete, static page.
   - One library per element (§8.1).
8. **Dependencies.** Only `gsap` and `motion` (P151), pinned exactly. No CDN, font host or new browser host. `web/vercel.json` does not change.
9. **Secrets.**
   - Never print, log, commit or paste a value from `.env`, `.env.deploy`, a token, a cookie or a link.
   - A script that uses `.env.deploy` prints names and pass or fail only, and lives in the scratchpad.
   - **Never query the production database.**
10. **Privacy.** Never stage `testclips/**`, `fixtures/.gen/**`, `corpus/**`, `web/test-results/**`, `.env*`, or anything from `~/design-review/` or `~/design-upload/`.
11. **Records.** Long runs start detached and are read from a log tail. Append to the changelog with the Edit tool, never with a heredoc that holds backticks.
12. **Branch.**
    - Commit on `design-build` only, and push only where the prompt says.
    - Never push to `main`, merge, tag or mark the pull request ready.
    - Never use Vercel's Redeploy button or Render's deploy hook.

**When existing code disagrees with the plan.** If it blocks the prompt, fix the minimum and record why. If it does not, report it under "Found, not fixed".

**The gate.** Run this prompt's tier: the new case alone first, then the rest. Then:

```bash
git branch --show-current
git status --porcelain | grep -c "zz[-_]"        # 0
git ls-files --cached | grep -E '(^|/)\.env'     # no output
# the frozen check of guide Step 0.2, with this prompt's exceptions: no output
```

Counts never fall below Setup's baseline.

Add a short entry to `designchangelog.md` in the same commit:

- what changed;
- what differs from the plan;
- **DECLARED CHANGE** lines;
- counts, and the app-shell size from P151 on;
- what is open;
- "Found, not fixed".

Rewrite its open-items table when an item opens or closes. Commit with the trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

Reply with:

- the files changed;
- the commands run and their results;
- deviations from the plan;
- **Human** steps still open;
- issues not fixed.

---

## Setup (design 0/10): baseline and draft pull request

**Effort: Medium.**

**Implement.** Guide Phase 0.

1. Check Step 0.1's branch and commits.
2. Run the frozen check (Step 0.2): it must be empty.
3. Record the baseline (Step 0.4) in DCL as Entry 3:
   - `pnpm check`;
   - `cargo test --workspace` (passed, failed, ignored);
   - `pnpm --filter web exec vitest run`;
   - `pnpm build`, and its `check-hosts` line;
   - `pnpm e2e`, with the counts per file.
4. Push `design-build` and open the draft pull request (Step 0.5). The body is written to a scratchpad file and ends with the session's attribution line.

**Acceptance.**
- [ ] Baseline in DCL Entry 3, including the app-shell size and the 18 landing cases passing.
- [ ] Draft PR open, CI started. CI's result is read before P151's push.
- [ ] No code changed.

---

## Prompt 151 (design 1/10): Foundations, copy, lint, nothing visible

**Effort: High** (it moves copy that every test reads).

**Read.** §3 (D-319, D-321, D-328, D-329), §4, §5, §6, §10.1-§10.4, §12.1; guide Phase 1; `web/src/copy/messages.ts`; `web/eslint.config.js`; `landing.spec.ts`; `scripts/check-file-tree.mjs`.

**Implement.**

1. **Dependencies.** Run `pnpm --filter web add -E gsap motion`. Read the licence text that `gsap` ships and quote its first paragraph and its restriction in the entry. If it restricts anything beyond products that compete with Webflow's visual builder, **stop and ask**.
2. **`web/src/copy/landing.ts`.** It exports `landingCopy` and `pricingCopy`.
   - **Moved without a change to any word:** every key of today's `messages.landing` and `messages.pricing`, except those of point 4.
   - **Added, with §6's drafts:**
     - the section copy;
     - `eyebrows` (the timecodes);
     - `features` (8 cards: `title`, `body`, `chips?`);
     - `soon` (the heading, `next` and `later` groups of `{ title, line }` with §5's 12 rows, `chip: "Coming soon"`, `footnote`);
     - `cta` (`startFree`, `howItWorks`);
     - `limits(LIMITS)`;
     - `playClip`;
     - `clipLabels` (one visually-hidden description per slot);
     - `marquee`;
     - `finalCall`;
     - `footerLine`;
     - `settingsLink: "Settings"`.
   - **Numbers** are function parameters (`LIMITS`, `PRICES_USD`), never digits in text.
3. **`messages.ts`.** It imports them: `landing: landingCopy`, `pricing: pricingCopy`. No caller changes.
4. **Words that change now:**
   - `landing.hero` becomes "Say it once. Post it finished." (F-1).
   - `landing.subhead` takes §6's draft.
   - `landing.waitlistHeading` is replaced by `landing.updatesHeading` ("Hear when the next features open."), and the old key is removed.
   - `landing.demoCaption` becomes "What Offcut makes: a raw take in, a finished short out." (D-324).
   - `notifyMe.label` becomes "Email me when new features open" (D-328).
   - `pricing.notOnSale` becomes "Start on Free with a free account. Creator goes on sale soon." (D-329, F-2 option (a)).
   - The `privacyLine`, `supportedBrowsers`, `whatLeavesLink`, `pricing.free/creator` features and prices, and `merchant` keep their words exactly.
5. **For now, `LandingPage.tsx` keeps rendering the old layout,** but the waitlist heading reads `updatesHeading`. That is the only line of it that changes.
6. **`eslint.config.js`:**
   - add `LANDING_CLIPS`, `LandingClip` and `LandingSlot` to the `ui → net/asset-fetch.ts` specifier list (their export comes in P156; listing them first is harmless);
   - add `no-restricted-imports` for `gsap`, `gsap/*`, `motion` and `motion/*` in `src/**` except `src/ui/landing/**`, with a message citing D-319.
7. **TS §5:** one row for every new file of §10.1 (including the `ui/landing/` folder's files, `copy/landing.ts`, the two CSS modules, the font, the posters folder and `public/og.png`). Also TS INV-15's wording (D-321).

**Tests.** Tier 2. All 18 landing cases pass **unchanged**: they read the moved copy through `messages`. A deliberate import of `gsap` in `ui/pages/LandingPage.tsx` fails lint; try it once, then remove it.

**Acceptance.**
- [ ] `messages.ts` is shorter than 679 lines; `copy/landing.ts` is under 700.
- [ ] `git grep -n "not open yet" web/src` shows only `legal` (the terms' "parts of it are not open yet", which stays).
- [ ] All 18 landing cases, signin and checkout pass, unchanged; Vitest and `pnpm check` pass.
- [ ] The GSAP licence is quoted in the entry.
- [ ] The frozen check is empty.

---

## Prompt 152 (design 2/10): The skeleton of the new page

**Effort: High.**

**Read.** §2, §6 (sections 0, 1, 6, 8, 10, 12), §7, §9.5, §11, §12.1; guide Phase 2; today's `LandingPage.tsx`, `DropZone.tsx`, `NotifyMeForm.tsx`, `components.module.css` (classes only), `tokens.css`.

**Implement.**

1. **`tokens.css`:**
   - add the `@font-face` "JetBrains Mono UI" (`../assets/jetbrains-mono-latin.woff2`, weight 700, `font-display: swap`);
   - add `:root:has([data-theme="dark"]) body { background: #0e0e11; }`;
   - rewrite the header comment (D-316).
   - Make the subset as guide Step 2 says, and add its row in `web/LICENSES.md` (OFL-1.1, size, SHA-256, source).
2. **`ui/styles/landing.module.css`:** the `.landing` root with §7.1's tokens, the layout of §7.5, and the top bar, hero and shared parts. **`landing-sections.module.css`:** the rest.
3. **`ui/landing/parts.tsx`:** `PhoneFrame`, `Chip`, `SectionHead` (timecode eyebrow, H2, lead), `CtaLink` (React Router `Link`, primary or secondary) and `BentoCard`.
4. **`ui/landing/LoopVideo.tsx`,** exactly as §9.5 says. In this prompt no slot has a clip, so it renders its children.
5. **`ui/landing/gsap.ts`:** `loadGsap()` (a cached dynamic import of `gsap` and `gsap/ScrollTrigger`, `registerPlugin`) and `useGsap(scopeRef, setup)` (`gsap.context` and `gsap.matchMedia` with `(prefers-reduced-motion: no-preference)`, reverted on unmount; a failed import is caught and leaves the page static). It is not called yet.
6. **`ui/pages/LandingPage.tsx`:**
   - The **same props**, and the **same navigation effect, character for character**.
   - The root is `<div className={styles.landing} data-landing data-theme="dark">`, inside `<LazyMotion features={domAnimation} strict><MotionConfig reducedMotion="user">`, and holds `<main>` with the sections.
   - Write these sections in this prompt:
     - `TopBar` (wordmark, anchors, "Start free", the progress-bar element);
     - `Hero` (H1 with `messages.landing.hero` as plain text for now, subhead, the two CTAs, the capability line moved verbatim with `role="status"` and `data-testid="capability-line"`, the facts with the one settings link, `PhoneFrame` with the `hero` slot);
     - `TryIt` (`<DropZone />` unchanged inside `.tryFrame`);
     - `Pricing` (today's content and test id, restyled; the Free card's link comes in P153);
     - `Updates` (`<NotifyMeForm wanted={["launch"]} />` unchanged);
     - `SiteFooter` (today's legal links and contact, `data-testid="site-footer"`, plus the `settingsLink`).
   - Restyle DropZone and NotifyMeForm only from `.tryFrame` and `.updates`, through attribute and element selectors (§11, bite 4).
7. The other sections come in P153.

**Tests.**

- Tier 2. **All 18 landing cases unchanged and passing.** This includes pricing, because pricing has no link yet.
- Write and pass L-3 (only the hero and top bar "Start free" for now; extend it in P153) and L-10.
- Tier 3: `rejections.spec.ts --grep "REJECT_FILE_SIZE"` on a keyed build, to show `/app` and the drop zone there are untouched.

**Acceptance.**
- [ ] `git diff "$BASE" -- web/src/ui/pages/LandingPage.tsx` shows the navigation effect's lines unchanged.
- [ ] No `style=` in `ui/landing/**`; no change in `components.module.css` or `pages.module.css`.
- [ ] The 18 cases, L-3 and L-10 pass; the frozen check is empty.
- [ ] The app-shell size is in the entry (it grows by Motion's `m` and `domAnimation`).

---

## Prompt 153 (design 3/10): Every feature, every "Coming soon", the way in

**Effort: High** (it holds the one declared test change).

**Read.** §4, §5, §6 (sections 2-5, 7, 9, 11, and the feature-card table), §12.1, §12.4, §13; guide Phase 3.

**Implement.**

1. **The static sections:**
   - `Marquee`;
   - `BeforeAfter`;
   - `Features` (the 8 cards of §6, each with its CSS illustration, and `LoopVideo` slots for `captionStyles` and `visuals` holding the illustration as children);
   - `HowItWorks`;
   - `Privacy` (its three facts, **not** the privacy line, and no second "What leaves your device" link);
   - `ComingSoon` (the "Next" and "On the way" groups, every card with the "Coming soon" chip, then the footnote);
   - `FinalCall`.
   - Each section has its `id` and its timecode eyebrow.
2. **Pricing** (D-329, option (a)): the Free card gets a `CtaLink` "Start free" to `appPath`; the Creator card gets its line, with no button and no link.
3. **Updates' heading:** `updatesHeading`.
4. **Responsive.** Read the page at 1280 × 800, 1920 × 1080 and 390 × 844. Nothing may scroll sideways (assert `document.documentElement.scrollWidth <= innerWidth` in L-1).

**DECLARED CHANGE (§12.4).** In "pricing: the plans show their prices and limits…":

- the title becomes "…and offer no way to buy, only a way in";
- `pricing.getByRole("link")` becomes a count of 1, with the name `landingCopy.cta.startFree` and `href` `/app`;
- every other line stays as it is. `pricing.getByRole("button")` still has a count of 0.

No other existing case is touched.

**Tests.**

- Tier 2. L-1 (each §4 card title, and no sideways scroll), L-2 (the soon count equals `landingCopy.soon` items, each with its chip), L-3 extended to all four "Start free" links, and L-4 (no "not open yet" or old waitlist sentence on `/`).
- Then go through §4's table row by row, and record in the entry where each row is named on the page.

**Acceptance.**
- [ ] 17 landing cases unchanged; the pricing case changed only as declared; L-1 to L-4 and L-10 pass.
- [ ] Every row of §4 marked for the page and every row of §5 is on the page (the entry's table).
- [ ] The frozen check is empty.

---

## Prompt 154 (design 4/10): Motion

**Effort: Medium.**

**Read.** §8 (rules 1-7; M5, M7, M8, M11), rule 2.5, bites 6 and 7.

**Implement.**

- **M5,** the hero chips: they enter in a stagger, then drift.
- **M7,** the section entrances: `whileInView`, `once`, `y` 16 → 0, and **no opacity on content**.
- **M8,** the bento spotlight: a pointer handler sets `--x` and `--y` through `style.setProperty` on the card's ref.
- **M11** in CSS: the waveform, the crop window, the chip pulse and the Creator border sweep (`@property --angle`). Every keyframe sits under `@media (prefers-reduced-motion: no-preference)`.
- **Never** use `layout`, `layoutId`, `AnimatePresence mode="popLayout"` or a `style` prop.

**Tests.**

- Tier 2.
- **L-5:** with `page.emulateMedia({ reducedMotion: "reduce" })`, after 1 s `document.getAnimations()` is empty on `/`; the chips are at rest.
- **L-7:** scroll top to bottom in 20 steps under `securitypolicyviolation` capture, and expect an empty list and `crossOriginIsolated`.
- The two `page.clock` cases pass, which shows content does not wait for frames.

**Acceptance.**
- [ ] `git grep -n "opacity: 0" web/src/ui/landing` hits decoration only; each hit is listed in the entry.
- [ ] L-5 and L-7 pass, and the earlier cases pass unchanged.
- [ ] The frozen check is empty.

---

## Prompt 155 (design 5/10): GSAP

**Effort: High.**

**Read.** §8 (M1, M2, M3, M4, M6, M9, M10), §10.3, D-317, D-318; guide Phase 5.

**Implement.** Everything goes through `useGsap()`, so it runs only after `loadGsap()` has resolved, inside `matchMedia("(prefers-reduced-motion: no-preference)")`, and is reverted on unmount.

1. **M1, `CaptionedHeadline`.**
   - It renders `<h1>` whose words are `<span>`s with **real text spaces** between them, so the accessible name stays `messages.landing.hero`.
   - It adds a pill `<span aria-hidden="true">` behind them.
   - The pill moves to each word by `x` and `width` measured with `getBoundingClientRect`, 0.28 s a word with the brand ease, and the word's colour switches. It plays once and rests on "finished". It measures again on resize.
   - Without GSAP: plain white text, no pill.
2. **M2:** the 2 px progress bar, `scaleX` scrubbed across the document.
3. **M3:** the playhead, only at 1280 px and wider. Each eyebrow gets the class `isLive` at `top center`.
4. **M4:** the hero phone's tilt from the pointer, up to 6° (`quickTo`), on fine pointers only.
5. **M6:** the marquee, which slows on hover.
6. **M9:** the How-it-works pin, only at 1024 px and wider; the steps advance with the progress.
7. **M10:** the final call's pill on its last word, once.

**Tests.**

- Tier 2.
- **L-8:** open `/app` directly, collect the body text of every `.js` response, and expect none to contain `ScrollTrigger`. Open `/`, scroll, and expect one to contain it.
- **L-9:** click the top bar's "Start free" (an in-app navigation), then expect no `.pin-spacer`, no `[data-landing]` and no `pageerror`.
- **J1** passes unchanged (the H1's name).
- The two `page.clock` cases pass.
- Then the DevTools Performance read of guide Phase 5: the longest task while scrolling, recorded.

**Acceptance.**
- [ ] The app shell is at most 500 kB; the total and the GSAP chunk alone are in the entry.
- [ ] L-7, L-8 and L-9 pass; all earlier cases pass unchanged.
- [ ] No task over 50 ms while scrolling on D1, or the entry names it and the fix.

---

## Prompt 156 (design 6/10): The clips

**Effort: High** (it publishes files).

**Read.** §9 (including the G-1 read of 2026-11-05), D-323 to D-326, D-335; guide Phase 6 (all of it); `scripts/upload-assets.sh`.

**Implement.**

1. **Re-encode** the seven web files from their masters into `~/design-upload/`, exactly as guide Step 6.1 says. Probe each again and record a table of format, fps, frames, audio, faststart and size in the entry. Read one mid frame of each beside the original.
2. **Posters** to `web/src/ui/assets/landing/<slot>.webp`, and `web/public/og.png` (Step 6.2).
3. **Human: the go.** Print the seven names and sizes and ask the founder: "Upload these seven files to the asset host's `media/` folder?" Upload only on a yes (Step 6.3). Never echo a key. Then check one path with `curl -sI` and an `Origin` header: expect 200, `video/mp4`, the immutable cache header and an `access-control-allow-origin` line.
4. **`net/asset-fetch.ts`:**
   - `export type LandingSlot = "hero" | "beforeAfter" | "captionStyles" | "visuals" | "howItWorks" | "privacy" | "comingSoon"`;
   - `LandingClip`;
   - `LANDING_CLIPS` with the seven printed paths and sizes;
   - `DEMO_VIDEO_PATH = LANDING_CLIPS.hero.path`, so the hand-made demo is referenced nowhere.
   - Each section passes `clip` and `poster` to `LoopVideo`.
5. **`web/index.html`:** title, description, `og:title`, `og:description`, `og:image` (the absolute URL of `/og.png` on the site's host), `og:type`, and `twitter:card` `summary_large_image`. No script, no external host.

**Tests.**

- Tier 2.
- **L-6:** for each slot, scroll it into view, then expect `readyState >= 2`, no `error` event, `paused === false` (normal motion), and `currentSrc` on the asset host.
- **`@smoke`** passes (the policy, isolation, WASM).
- "Every request goes to the page's own origin or to the asset host" passes unchanged.
- The unsupported page shows the hero clip (Playwright's mobile emulation).

**Acceptance.**
- [ ] Seven clips uploaded with the founder's go; their paths are in the entry, and no key is printed anywhere.
- [ ] `git grep -n "demo.81cafa494d9c2b68" web/src` is empty.
- [ ] L-6 passes; all earlier cases pass unchanged; `check-hosts.mjs` is ok.
- [ ] Nothing from `~/design-upload/` is staged.

---

## Prompt 157 (design 7/10): The other pages on the dark system (Part C)

**Effort: Medium.**

**Read.** §15; guide Phase 7; `pages.module.css`; the six pages' roots.

**Implement.** Add `data-theme="dark"` to the root element of `SettingsPage`, `LegalPage`, `AccountPage`, `AuthCallbackPage`, `UnsupportedPage` and `CapabilityGate`'s checking `<main>`. In `pages.module.css`, use the dark tokens: `--bg`, `--surface-1`, `--border`, `--text`, `--text-2`, `--brand`, `--ok` and `--danger`, in place of V1's `--color-*`. The danger button, dialog and backdrop keep their roles. The font becomes Inter UI.

**No markup, copy or logic change.** The diff of each `.tsx` is one attribute.

**Frozen-check exceptions:** these six `.tsx` files and `pages.module.css`.

**Tests.**

- Tier 2 (signin and checkout read the account page).
- Tier 3: `checkout.spec.ts` in full is already non-media. Add one media case that ends on the account page or the unsupported page, if one exists; otherwise record that none does.
- **L-12:** `axe(page)` on `/settings` and `/legal/terms` returns an empty list.

**Acceptance.**
- [ ] Each page `.tsx` diff is the `data-theme` attribute only.
- [ ] signin, checkout and every landing case pass unchanged; L-12 passes.
- [ ] Contrast: the account page's danger button and status lines are AA under axe.

---

## Prompt 158 (design 8/10): axe, the captures, design review 3

**Effort: Medium.**

**Read.** §12.3 (L-11), §12.5, D-333; guide Phase 8; `tests-e2e/screens.spec.ts`; `helpers/fixtures.ts` (`axe`).

**Implement.**

1. **L-11:** `axe(page)` on `/` at the top and after scrolling to the end, expecting an empty list. Fix what it finds in CSS or markup; never by turning a rule off.
2. **`screens.spec.ts`:** add a test "landing" that captures `/` full page at 1280 × 800, 1920 × 1080 and 390 × 844, under normal and reduced motion. The landing needs no media, but the project's dependency stays as it is.
3. Run `pnpm screens`. Copy `web/test-results/screens/landing-*` to `~/design-review/`.
4. **Human: design review 3.**
   - The founder reads the captures and the page (`pnpm build && pnpm --filter web exec vite preview`, then `http://localhost:4173/`), with the creator friend if possible.
   - The founder reads every sentence of `copy/landing.ts`.
   - They answer "go" or give a list of changes. Make the changes, run Tier 2 again, and quote the founder's "go" in the entry.

**Acceptance.**
- [ ] L-11 is empty at both positions.
- [ ] The captures are in `~/design-review/`, none staged.
- [ ] The founder's "go" is quoted. Every requested change is made and listed.

---

## Prompt 159 (design 9/10): Lower Bottom captions (Part B, DC-13)

**Effort: High** (it changes protected goldens under a declared row).

**Read.**

- §14 (all of it), TS §41.1 and §41.2.
- `crates/offcut-scene/src/captions.rs` (`lines()`, line 367), `placement.rs`, `safe_area.rs`, `styles.rs` (`decoration_width`).
- `crates/offcut-render/tests/golden_frames.rs` (`OFFCUT_GOLDEN_UPDATE`).
- `crates/offcut-scene/tests/display_list_snapshots.rs`, `layout_safe_area.rs`, `hit.rs`.
- Guide Phase 9.

**Implement.**

1. **Stills first** (guide Step 9.1). In a working copy, with option A behind a local constant that is not committed:
   - render the reference frames for Clean, Bold and Mono, one and two lines, today and A, at 1080 × 1920;
   - render the Free frame with the watermark under A.
   - Save them to `~/design-review/captions/`.
   - Look at each. If A puts any line or pill on the watermark, or outside the safe area, or reads worse than today in any style, **stop and ask**.
2. **Code** (`captions.rs`).
   - Replace the centring `top = block.y + (block.h - height * n) / 2.0` with an alignment by the block's place in the band:

     ```text
     align = clamp((block centre - CAPTION_TOP) / (CAPTION_BOTTOM - CAPTION_TOP), 0, 1)
     ```

     This is 0 at the top, 1 at Bottom and 0.5 halfway, through a `pub(crate)` helper in `placement.rs`, so `lines()` keeps its signature. The new expression is `top = block.y + inset + (block.h - 2·inset - height · n) · align`.
   - `inset` keeps the style's vertical decoration (Clean's pill padding, Mono's panel padding, the stroke) inside the block. Read the actual values from the style, and do not invent them.
   - At placement 0.5 the output equals today's, bit for bit (a unit test).
   - `hit.rs` reads the same lines, so hit-testing follows. `CaptionPlacement`, the block, the event zone, the safe area and the watermark do not change.
3. **Unit tests in `captions.rs`:**
   - placement 0.5 is unchanged;
   - at 1.0 the last line's bottom (with its decoration) is at most the block's bottom, and at least `block.bottom - 2` lp;
   - at 0.0 the first line's top is at the block's top plus `inset`.
4. **Snapshots.** Run `INSTA_UPDATE=always cargo test -p offcut-scene` (or `cargo insta review`). Read every changed `.snap`: only `y` and `baseline` fields of glyph runs, pills and panels may differ. Record the files and fields in the entry.
5. **Goldens.**
   - Run `OFFCUT_GOLDEN_UPDATE=1 cargo test -p offcut-render --test golden_frames`.
   - Run it again without the variable; it must pass.
   - Look at all 16 PNGs, and record how many changed.
   - Today's parity case must still pass.
6. **TS §41.2:** add the DC-13 row of §14.3, with the date and "founder's choice: option A, 2026-11-05".

**Frozen-check exceptions:** `crates/offcut-scene/src/captions.rs`, `crates/offcut-scene/src/placement.rs` (the helper only), the scene's `tests/snapshots/**`, `fixtures/golden/*.png` and `docs/technicalspec.md`. `verify/**` and the watermark are untouched.

**Tests: Tier 4, once.**

- `cargo test --workspace`.
- Vitest.
- `pnpm build` and `pnpm e2e`.
- Then `pnpm build:e2e` and the **full** `pnpm e2e:media`, detached, read from a log tail. `export-free` and `export-creator` must pass all 11 checks of `verify_mp4.py`; `preview-sync` and `review-edit` (including the drag cases) must pass unchanged.

**Acceptance.**
- [ ] The stills were looked at; A held; the paths are in the entry.
- [ ] Placement 0.5 is unchanged, bit for bit; only the declared fields differ in the snapshots.
- [ ] Every count is at or above Setup's baseline, plus the new unit tests; the full media run is green.
- [ ] DC-13 is in TS §41.2. **DECLARED CHANGE** line in the entry.

---

## Prompt 160 (design 10/10): Copy-back, merge, deploy checks, tag

**Effort: Medium.**

**Read.** §19, §20; guide Phase 10.

**Implement.**

1. Copy back §20 into TS, PS, BP and the V6 changelog's open item 10 (the resubmission row). TS §39.3 item 24 becomes 500 kB, with this release's reading. Add TDR-26.
2. Push; `gh pr checks <pr> --watch` in the background. One CI run for the batch.
3. **Human:** `gh pr ready`, merge, and watch the `main` run (Step 10.3).
4. After the deploy:
   - `E2E_BASE_URL=https://offcut-one.vercel.app pnpm --filter web exec playwright test --project=non-media --grep @smoke`;
   - read `/` on D1 in Chrome and in Edge: every clip plays, the headline animates, reduced motion is static;
   - `curl -sI` the site's `/` shows the unchanged security policy.
5. **Human:**
   - resubmit to Dodo with the live page (V6 CL open item 10);
   - tag `design` (Step 10.6).
6. Write the closing entry: every box of §19 ticked or named; open item 17 (the in-app offers, F-3) carried as its own change.

**Acceptance.**
- [ ] CI green on the pull request and on `main`; `@smoke` green against the deployment.
- [ ] §20 copied back.
- [ ] §19 ticked or each open box named.
- [ ] **Human:** the merge, the Dodo resubmission and the tag `design`.

---

# Forbidden shortcuts

| Shortcut | Why not |
|---|---|
| Edit an existing landing, signin, checkout or media case to make it pass (except §12.4's pricing case) | The founder's rule: a failing case means the code broke something |
| Repeat the privacy line, "Try a sample", "Notify me" or the "What leaves your device" link anywhere else on `/` | Four existing cases fail on two matches (bite 1) |
| Restyle by editing `components.module.css` or `pages.module.css` in Part A | Shared with the sign-in dialog, the unsupported page and five other pages (bites 4, 5) |
| A second `<form>` on `/` | The waitlist cases read the first form's status (bite 3) |
| `opacity: 0` as the initial state of content | Content stays hidden under `page.clock`, a failed chunk or a slow device (rule 2.5) |
| `style=` in JSX, `setAttribute("style")`, `<style>`, `popLayout` | The security policy blocks it; the SAB forbids it |
| Add `'unsafe-inline'` or a host to `vercel.json` | The policy is a protected promise (PS §17) |
| Lazy-load `LandingPage` in `routes.tsx` | Changes the first paint and the router; only GSAP is lazy (bite 12) |
| Import `gsap` or `motion` outside `ui/landing/` | D-319: the editor stays free of them |
| Type 90, 3, 15 or 144 into a sentence | TS §11.3: numbers come from `LIMITS` and `PRICES_USD` |
| Upload `offcut-remotion/out/` as it is | Full-range `yuvj420p`; re-encode first (bite 13) |
| Upload without the founder's go, or print an `ASSET_S3_*` value | It publishes files and handles a secret |
| Regenerate goldens or snapshots outside P159 | They change only under DC-13 |
| Move the watermark or the safe area for the captions | It changes the verifier's mask, a protected proof (§14.1); that is option C, which was not chosen |
| A new analytics event for CTA clicks | The allowlist, the server enum and "What leaves your device" would change (§1.2) |
| Claim a post-V6 feature without "Coming soon", or anything about other platforms | Rule 2.4 |
