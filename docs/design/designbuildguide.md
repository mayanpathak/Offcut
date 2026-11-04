# designbuildguide.md - Offcut design release: step-by-step build guide

## What this is

`docs/design/designimplementation.md` says what each file of the design release holds and why. This guide says, for each prompt:

- what to do next;
- which command to run;
- how to know it worked.

The release is built on `design-build`, with a draft pull request into `main` so that CI runs. It ends with:

- the pull request merged;
- the new landing page live, every box of §19 ticked;
- the tag `design` pushed by the founder.

**The founder's rule for this release: nothing that works today may break.** Every prompt proves it twice, before and after its change. The prompts change no logic outside the files §10.1 names.

## How to read the references

| Notation | Means |
|---|---|
| `§n` | Section `n` of `designimplementation.md` |
| `D-n` | A decision: D-316 to D-335 belong to this release |
| `L-n` | A new test case of `landing.spec.ts` (§12.3) |
| `M-n` | An animation (§8.2) |
| `P<nnn>` | A prompt of `coding-promptsdesign.md` (151 to 160) |
| `TS`, `PS`, `BP` | `technicalspec.md`, `product.md`, `buildplan.md` |
| DCL | `docs/design/designchangelog.md` |
| D1 | The founder's Windows machine |
| "the founder" | The person running the build. Uploads to the asset host, merges, tags and the Dodo resubmission are theirs, or need their go |

## Phase overview

| Phase | Prompt | Covers | Agent time (estimate) | Milestone in one line |
|---|---|---|---|---|
| 0 Setup | Setup | §16 | 45 min | Baseline counted; draft PR open |
| 1 Foundations | 151 | D-319, D-321, §10.1-§10.4 | 1.5 h | Dependencies, copy file, lint rules; nothing on screen changed |
| 2 Skeleton | 152 | §6 (0, 1, 6, 8, 10, 12), §7 | 3 h | The new page's frame, with all 18 old cases green |
| 3 Sections | 153 | §6 (2-5, 7, 9, 11), D-328, D-329 | 3 h | Every feature and every "Coming soon" on the page; the declared pricing change |
| 4 Motion | 154 | §8 (M5, M7, M8, M11) | 2 h | React-side animation; reduced motion static |
| 5 GSAP | 155 | §8 (M1-M4, M6, M9, M10) | 3 h | The captioned headline, progress bar, playhead, pin; GSAP never on `/app` |
| 6 Clips | 156 | §9, D-323 to D-326, D-335 | 2 h + founder's go | Seven clips playing under the isolation headers |
| 7 Other pages | 157 | §15 | 1.5 h | Settings, legal, account, callback and unsupported on the dark tokens |
| 8 Review | 158 | L-11, §12.5 | 1.5 h + founder | axe clean; captures; design review 3 says "go" |
| 9 Captions | 159 | §14 | 3 h | DC-13: lower Bottom captions; goldens regenerated; one full media run |
| 10 Close | 160 | §19, §20 | 1.5 h + founder | Copy-back, merge, deploy checks, tag |

Total: about 22 hours of agent time over 4-6 working days, plus the founder's steps. All are estimates.

## Read this first: 16 things that will bite you

1. **Playwright matches text by substring, and a match of two elements fails.**
   - `getByText(messages.landing.privacyLine)` fails if a second element anywhere on `/` contains "Your video stays on your computer.".
   - `getByRole("link", { name: whatLeavesLink })` fails if the footer also has a "What leaves your device" link.
   - The list is §12.1. Check it before any copy change.
2. **"Try a sample" must stay on the drop zone's button only.** A hero button with that name breaks J1 and a media case that clicks it on `/app`.
3. **One `<form>` on `/`.** `formStatus` is `page.locator("form").getByRole("status")`. A second form, a search box for example, breaks the four waitlist cases.
4. **`DropZone`'s and `NotifyMeForm`'s classes are shared** with `SignInDialog` and the unsupported page. Do not edit `components.module.css`. Style them from the landing's scope with attribute and element selectors (§11).
5. **`pages.module.css` is shared by six pages.** Part A does not touch it. The new page uses `landing.module.css` and `landing-sections.module.css`. Part C (P157) is the only prompt that changes `pages.module.css`.
6. **`page.clock.install()` freezes `requestAnimationFrame`.** Two existing cases (`landing_view`, `capability_check`) install it, so GSAP and Motion stand still there. That is harmless only because content is never hidden by an initial state (rule 2.5). Decoration may start at opacity 0; content never does.
7. **The security policy has no `'unsafe-inline'` for styles.**
   - `element.style.x = …` and `style.setProperty` are allowed: they go through the CSS object model.
   - `setAttribute("style", …)`, a `<style>` element and Motion's `AnimatePresence mode="popLayout"` are not allowed.
   - There is no `style=` in JSX either: the SAB forbids inline style.
   - `@smoke` and L-7 catch a violation; do not "fix" one by editing `vercel.json`.
8. **`ui` may import only named exports of `net/asset-fetch.ts`.** `assetUrl` and `DEMO_VIDEO_PATH` are allowed today. `LANDING_CLIPS` and the types are added to that list in `eslint.config.js` (P151) before any section imports them.
9. **Every new file needs its TS §5 row in the same commit.** Otherwise `check-file-tree.mjs` fails `pnpm check`.
10. **`messages.ts` is at 679 of 700 lines.** Landing and pricing copy move to `copy/landing.ts` (D-321) before any copy is added.
11. **No digit typed into a sentence** (TS §11.3). 90 seconds, 3 exports and the prices come from `LIMITS` and `PRICES_USD`. "720p" and "1080p" are written as `pricing` already writes them.
12. **`routes.tsx` imports `LandingPage` statically. Keep it so.** Making it lazy changes the first paint and the router's behaviour. Only GSAP is loaded by `import()`.
13. **The clips are `yuvj420p` (full range).** P156 re-encodes them to `yuv420p` limited range before uploading (Phase 6). Upload the re-encoded files, never `out/` as it is.
14. **The upload uses the asset host's write keys.** They come from `.env.deploy` inside a subshell. Never print or echo them. The founder gives the go before anything is uploaded: uploading publishes the files.
15. **Goldens and snapshots change only in P159**, under DC-13:
    - goldens with `OFFCUT_GOLDEN_UPDATE=1`;
    - snapshots with `cargo insta review` (or `INSTA_UPDATE=always`, then reading every diff).
    - A golden or `.snap` diff in any other prompt is a regression.
16. **Test counts only rise** (TS §41.1). Setup records the baseline; every entry records its counts.

---

## Phase 0 - Setup

### Step 0.1 - The branch (done)

`design-build` exists, starting at `8d94a06`, with the plan committed as `e50f753`. Check it:

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut
git fetch origin --tags --prune
git branch --show-current            # design-build
git log --oneline -3                 # the plan commits on top of 8d94a06
git log --oneline origin/main..8d94a06   # one line: V6's closing entry
```

### Step 0.2 - The frozen check

`BASE=8d94a06`. These paths must not change in Parts A and C:

```bash
git diff --name-only "$BASE" -- crates server verify fixtures \
  web/src/usecases web/src/state web/src/workers web/src/persistence web/src/models \
  web/src/platform web/src/analytics web/src/entitlement web/src/wasm web/src/gen \
  web/src/net/http.ts web/src/net/api-client.ts web/src/config \
  web/src/main.tsx web/src/App.tsx web/src/routes.tsx web/vercel.json \
  web/src/ui/components web/src/ui/layout web/src/ui/primitives web/src/ui/hooks \
  web/src/ui/pages/EditorPage.tsx \
  web/src/ui/styles/components.module.css web/src/ui/styles/editor.module.css \
  web/src/ui/styles/panel.module.css web/src/ui/styles/preview.module.css \
  web/src/ui/styles/primitives.module.css web/src/ui/styles/controls.module.css
# expect: no output, in every prompt before P159
git diff --name-only "$BASE" -- '*.snap' fixtures/golden
# expect: no output, in every prompt before P159
```

P157 (Part C) adds a small set of page files (§15). P159 (Part B) adds `crates/offcut-scene/src/captions.rs`, the snapshots and the goldens. Each prompt names its exceptions, and the check is run with them.

### Step 0.3 - The tiers

| Tier | Runs | Used by |
|---|---|---|
| 0 | The new or changed case alone: `pnpm build && pnpm --filter web exec playwright test --project=non-media landing.spec.ts --grep "<case>"` | Every prompt, first |
| 1 | `pnpm check`; `pnpm --filter web exec vitest run` | Every prompt |
| 2 | Tier 1, `pnpm build` (it runs `check-hosts.mjs` and prints the shell size), `pnpm e2e` (landing, signin, checkout: the 18 + new landing cases and the 2 other files) | 151 to 158, 160 |
| 3 | Tier 2, plus the named media cases on a keyed build: `pnpm build:e2e && pnpm e2e:media <spec> --grep "<case>" --max-failures=1` | 152 (one drop-zone case), 157 |
| 4 | Tier 2, `cargo test --workspace`, and the full `pnpm e2e:media` on a keyed build, detached, once | 159 |

Run targeted tests first; the full media run happens once (P159). Media suites need `pnpm build:e2e` (the test mode) and Docker is not needed.

### Step 0.4 - Baseline (Setup)

Record in DCL Entry 3:

- `pnpm check`: pass.
- Rust: `cargo test --workspace`, the passed, failed and ignored counts.
- Vitest: the count.
- `pnpm e2e`: the count (expect 18 landing + signin + checkout).
- App shell, gzip: the `check-hosts.mjs` line after `pnpm build` (expect 351.6 kB).
- The media floor: V6's closing counts (V6 CL), not re-run now.

### Step 0.5 - Draft pull request (Setup)

```bash
git push -u origin design-build
gh pr create --draft --base main --head design-build --title "Design release: landing page, clips, lower captions" --body-file <scratchpad file>
```

The body ends with the attribution line of the session. Nobody marks it ready except the founder.

**Milestone 0.** Baseline in DCL, draft PR open, frozen check empty.

---

## Phase 1 - Foundations (P151)

| Step | What | Check |
|---|---|---|
| 1.1 | `pnpm --filter web add -E gsap motion`. Read `node_modules/gsap/README.md` and the licence text the package ships; quote its first paragraph and the restriction in DCL | The restriction concerns Webflow-competing visual builders only; otherwise stop (D-319, §10.2) |
| 1.2 | `copy/landing.ts`: `landingCopy` (hero, facts, sections 0-12, the 8 feature cards, the 12 soon items, `playClip`, `demoCaption`) and `pricingCopy`, with the drafts of §5 and §6 and F-1's H1. `messages.ts`: `landing: landingCopy, pricing: pricingCopy`, `notifyMe.label` reworded (D-328) | `messages.landing.supportedBrowsers` etc. resolve unchanged; `wc -l messages.ts` goes down |
| 1.3 | `eslint.config.js`: `LANDING_CLIPS`, `LandingClip`, `LandingSlot` added to the `asset-fetch.ts` specifiers; `no-restricted-imports` for `gsap*` and `motion*` outside `src/ui/landing/**` | `pnpm check` passes; a test import of `gsap` in `ui/pages/` fails lint (try it, then remove it) |
| 1.4 | TS §5 rows for every file of §10.1, in one edit; INV-15's wording (D-321) | `node scripts/check-file-tree.mjs` |

At the end of P151 the landing page still renders the old layout, with words that differ only where §6 says. **Proof:** Tier 2. All 18 landing cases still pass, because they read the copy through `messages`.

---

## Phase 2 - Skeleton (P152)

| Step | What | Check |
|---|---|---|
| 2.1 | `tokens.css`: `@font-face` for JetBrains Mono UI (subset: see below), the `:has()` body rule, the header comment (D-316) | `pnpm build`: the woff2 under `dist/assets/` |
| 2.2 | `landing.module.css`, `landing-sections.module.css`, `parts.tsx` (`PhoneFrame`, `Chip`, `SectionHead`, `CtaLink`, `BentoCard`) | No file over 700 lines |
| 2.3 | `LoopVideo.tsx` (§9.5), with `clip` undefined everywhere for now, rendering its children | No `<video>` on `/` yet |
| 2.4 | `gsap.ts`: `loadGsap()` and `useGsap()`, unused yet | Tier 1 |
| 2.5 | `LandingPage.tsx`: the root `<div data-landing data-theme="dark">`, `LazyMotion` and `MotionConfig`, then TopBar, Hero, TryIt, Pricing (today's content), Updates and SiteFooter. `capabilityLine` moved verbatim; the navigation effect untouched | The diff of the effect's lines is empty (`git diff -U0` shows only the JSX and imports) |

**The font subset.** Run `pyftsubset` (fonttools, as V4 did for Inter) on `crates/offcut-scene/assets/fonts/JetBrainsMono-Variable.ttf`:

- `--unicodes="U+0020-007E,U+00B7,U+00D7,U+2192,U+2014"`
- `--flavor=woff2`
- `--layout-features=""`
- pin `wght` to 700 with `fonttools varLib.instancer` first.

Record the size and SHA-256 in `web/LICENSES.md`.

**Proof:**

- Tier 2.
- Tier 3 for one media case that starts from `/app`'s drop zone (`rejections.spec.ts --grep "REJECT_FILE_SIZE"`), to show `/app` is untouched.
- L-3 and L-10 written and passing.

---

## Phase 3 - Sections (P153)

| Step | What | Check |
|---|---|---|
| 3.1 | Marquee, BeforeAfter, Features (8 cards), HowItWorks, Privacy, ComingSoon, FinalCall, static, with CSS illustrations in the clip slots | Each feature row of §4 marked "on the page" has its words on `/` |
| 3.2 | Pricing: the Free card's "Start free" link, Creator's line (option (a)), and the `notOnSale` words (D-329) | The pricing case changed exactly as §12.4 says: **DECLARED CHANGE** in the entry |
| 3.3 | Updates: the heading (D-328) | The four waitlist cases unchanged and passing |
| 3.4 | L-1, L-2, L-4 | Pass |

**Proof:** Tier 2. Read the whole of `/` at 1280 × 800 and 390 × 844, and record any section that overflows sideways (none allowed).

---

## Phase 4 - Motion (P154)

Motion's work: M5 (the hero chips), M7 (section entrances), M8 (the spotlight), and M11 in CSS (waveform, crop window, chip pulse, Creator border sweep), all under `(prefers-reduced-motion: no-preference)`.

**Check that nothing is hidden.** No `initial` with `opacity: 0` on an element whose text is content. Run:

```bash
git grep -n "opacity: 0" web/src/ui/landing
```

Read every hit against rule 2.5.

**Proof:**

- Tier 2.
- L-5 (reduced motion) and L-7 (scroll, zero violations).

---

## Phase 5 - GSAP (P155)

GSAP's work: M1, M2, M3, M4, M6, M9 and M10, each inside `useGsap()` (context and `matchMedia`, reverted on unmount).

| Check | How |
|---|---|
| The H1's accessible name is unchanged | J1 passes; `getByRole("heading", { level: 1 })` has `messages.landing.hero` as its name |
| No GSAP on `/app` | L-8 |
| Nothing left after leaving `/` | L-9 |
| No long task while scrolling | DevTools Performance on D1, a scroll from top to bottom of a production build: no task over 50 ms; record the longest |
| The size | `pnpm build`: the shell total, and the GSAP chunk's own gzip size (`gzip -c dist/assets/<chunk>.js \| wc -c`); both in the entry |

**Proof:**

- Tier 2.
- The two `page.clock` cases pass, which shows content is not hidden when frames stop.

---

## Phase 6 - Clips (P156)

### Step 6.1 - Re-encode (agent)

The source is `C:\Users\Mayan\Desktop\sh2clips\offcut-remotion\out\` (read only). The target is `~/design-upload/`, outside both repositories. For each clip, encode from its **master** to the web size:

```bash
ffmpeg -v error -y -i "$OUT/offcut-hero-1080x1920.mp4" \
  -vf "scale=720:1280:flags=lanczos:in_range=pc:out_range=tv,format=yuv420p" \
  -color_range tv -colorspace bt709 -color_primaries bt709 -color_trc bt709 \
  -c:v libx264 -profile:v high -preset slow -crf 20 -r 30 -an -movflags +faststart \
  ~/design-upload/offcut-hero-720x1280.mp4
```

The same goes for before-after (1280x720), caption-styles (720x1280), visuals (960x960), how-it-works (1280x720), privacy (1280x720) and coming-soon (1280x720).

Then run the probe of §9.2 again on the seven files:

- `yuv420p`;
- 30 fps;
- no audio;
- faststart;
- under the cap;
- the frame count unchanged (300/360/270/360/360/180/240).

Read one mid frame of each beside the original, to check the colours look the same.

### Step 6.2 - Posters

1. Copy the seven `.webp` posters to `web/src/ui/assets/landing/<slot>.webp`, using the slot names of §9.1.
2. Make `web/public/og.png` (1200 × 630): the hero poster on `#0e0e11`, centred, made with ffmpeg; under 300 kB.
3. Add `og.png` to TS §5.

### Step 6.3 - Upload (founder's go)

The agent prints the seven file names and sizes and asks the founder: "Upload these seven files to the asset host's `media/` folder?" Only on a yes:

```bash
grep -c '^ASSET_S3_' .env.deploy        # expect 4 or 5 names; prints a number only
( set -a; . ./.env.deploy; set +a; sh scripts/upload-assets.sh media ~/design-upload/*.mp4 )
```

The script prints the seven hashed paths and nothing else. If the names are missing, stop: the founder runs the same command in their own shell and pastes the paths.

**Check the upload:**

```bash
curl -sI -H "Origin: https://offcut-one.vercel.app" "<base>/<path>"
```

The response must show `200`, `video/mp4`, the immutable cache header and `access-control-allow-origin`.

### Step 6.4 - Code

| Step | What |
|---|---|
| 6.4.1 | `asset-fetch.ts`: `LandingSlot`, `LandingClip`, `LANDING_CLIPS` with the seven paths; `DEMO_VIDEO_PATH = LANDING_CLIPS.hero.path` (the hand-made demo's path stays in a comment of the changelog, not the code) |
| 6.4.2 | Each section passes its clip and poster to `LoopVideo` |
| 6.4.3 | `index.html`: the title, the description, `og:` and `twitter:` tags, with an absolute `og:image` URL on the site's own host (D-335) |
| 6.4.4 | `demoCaption`'s new words (D-324) |

**Proof:**

- Tier 2.
- L-6.
- `@smoke`.
- The unsupported page, opened in a browser without WebGPU or with the mobile emulation of Playwright, shows the hero clip.
- `check-hosts.mjs` ok.

---

## Phase 7 - Other pages (P157, Part C)

`data-theme="dark"` goes on the roots of `SettingsPage`, `LegalPage`, `AccountPage`, `AuthCallbackPage` and `UnsupportedPage`, and on `CapabilityGate`'s checking state. `pages.module.css`'s V1 variables map to the dark tokens. There are no markup, copy or logic changes.

**Frozen-check exceptions:** those six files and `pages.module.css`.

**Proof:**

- Tier 2.
- Tier 3: `signin.spec.ts` and `checkout.spec.ts` (non-media, already in Tier 2), plus one media case that reaches the unsupported page or the account page.
- L-12: axe on `/settings` and `/legal/terms`.

---

## Phase 8 - Review (P158)

| Step | Who | What |
|---|---|---|
| 8.1 | Agent | L-11 (axe on `/`), fixes |
| 8.2 | Agent | `screens.spec.ts`: a `landing` test at 1280 × 800, 1920 × 1080 and 390 × 844, full page, under normal and reduced motion. `pnpm screens`, then copy `web/test-results/screens/landing-*` to `~/design-review/` (never staged) |
| 8.3 | Agent | A 20-second screen recording of the scroll is optional. If made, it goes in `~/design-review/` only |
| 8.4 | **Founder** | Design review 3: the founder, with the creator friend if possible, reads the captures and the page from `pnpm --filter web exec vite preview`. They read every sentence of `copy/landing.ts` (D-333). They answer "go" or a list of changes. The changes are made in P158 before its commit |

**Milestone 8.** The founder's "go" is quoted in DCL.

---

## Phase 9 - Captions (P159, Part B, DC-13)

| Step | What | Check |
|---|---|---|
| 9.1 | A development build with option A behind a local constant (not committed); 18 stills (3 styles × 1 and 2 lines × today, A, and the watermark frame at Free) to `~/design-review/captions/` | The founder looks; A as decided, unless it reads worse (then stop) |
| 9.2 | `captions.rs`: `top = block.y + (block.h - H) * align` in place of `/ 2.0`, with `align` derived from the block's position in the band (0 at `CAPTION_TOP`, 1 at `CAPTION_BOTTOM`, 0.5 in the middle), so `lines()` keeps its signature, and the style's outermost decoration stays inside the block | `cargo test -p offcut-scene`: the failures are exactly the placement-dependent snapshots, and `layout_safe_area.rs` still passes |
| 9.3 | Snapshots: `cargo insta test -p offcut-scene --review` or `INSTA_UPDATE=always`; read every changed `.snap`. Only `y`/`baseline` values of glyph runs, pills and panels may differ | A diff summary in the entry: files, fields, nothing else |
| 9.4 | Goldens: `OFFCUT_GOLDEN_UPDATE=1 cargo test -p offcut-render --test golden_frames`, then run it again without the variable | The second run passes; 16 PNGs reviewed by eye; the count changed is recorded |
| 9.5 | TS §41.2: the DC-13 row (§14.3), with the date and the founder's choice | |
| 9.6 | Tier 4, once, detached; `verify_mp4.py`'s 11 checks via `export-*.spec.ts` | Counts at or above the baseline |

**Frozen-check exceptions:** `crates/offcut-scene/src/captions.rs`, `crates/offcut-scene/tests/snapshots/**`, `fixtures/golden/*.png`, `docs/technicalspec.md`. `verify/` stays untouched.

---

## Phase 10 - Close (P160)

| Step | Who | What |
|---|---|---|
| 10.1 | Agent | Copy-back §20. `git grep -n "TODO\|FIXME" web/src/ui/landing` is empty |
| 10.2 | Agent | Push; `gh pr checks <pr> --watch` in the background; one CI run for the batch |
| 10.3 | **Founder** | `gh pr ready`, merge, and watch the `main` run (Render redeploys only the API image, which is unchanged; Vercel builds the site) |
| 10.4 | Agent | After the deploy: `E2E_BASE_URL=https://offcut-one.vercel.app pnpm --filter web exec playwright test --project=non-media --grep @smoke`; read `/` on D1 in Chrome and Edge; L-6 by eye (every clip plays) |
| 10.5 | **Founder** | Resubmit to Dodo with the live page (V6 CL open item 10). Open item 17 (the in-app offers) stays for its own change (F-3) |
| 10.6 | **Founder** | `git tag -a design -m "Design release: landing page, clips, lower captions" <merge> && git push origin design` |

**Milestone 10.** §19 is ticked or each open box is named.

---

## Quick reference

```bash
# Every terminal: Git Bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
BASE=8d94a06
pnpm check                                   # every static check, including the file tree
pnpm --filter web exec vitest run
pnpm build                                   # production build + check-hosts + shell size
pnpm e2e                                     # landing, signin, checkout
pnpm --filter web exec playwright test --project=non-media landing.spec.ts --grep "L-5"
pnpm build:e2e && pnpm e2e:media rejections.spec.ts --max-failures=1
OFFCUT_GOLDEN_UPDATE=1 cargo test -p offcut-render --test golden_frames   # P159 only
```
