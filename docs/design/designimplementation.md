# designimplementation.md - Offcut design release: the landing page, the clips, lower captions

**Implements:** the founder's design release of 2026-11-04. It sits between V6 (closed, tag `v6`) and the cloud-transcription release that comes before V6.1. It covers three things:

- a new landing page, animated with GSAP and Motion;
- the Remotion clips that replace the hand-made demo video;
- a lower "Bottom" caption position.

**Why.** A creator who reviewed the live site called the landing page "extremely basic" and "not at all appealing". Dodo Payments put its product review on hold because the page reads as a waitlist (V6 CL open item 10). Creators also said that the Bottom captions look as if they sit in the middle.

**References.** "PS" is `product.md`, "TS" is `technicalspec.md`, "BP" is `buildplan.md`, "V6 CL" is `docs/v6/v6changelog.md` and "DCL" is `docs/design/designchangelog.md`. Decisions D-1 to D-315 belong to V1-V6; this document continues at **D-316**.

**Reader.** Whoever writes the code of this release. A build guide and coding prompts follow once the founder approves this file, and this file wins where they disagree.

**Read on.** The code at `8d94a06` (`v6-build`, which is `origin/main` plus V6's closing entry) on 2026-11-04. Every "today" below is a fact of that commit.

**Markers:**

- **(founder)**: a choice that waits on the founder, with my recommendation.
- **(assumption)**: a number nobody has measured.
- **DECLARED CHANGE**: a change to something a test or a contract holds. Each one is listed in §12.4.

**The one-sentence goal.** A visitor sees in five seconds what Offcut does: what goes in, what comes out, that it runs on their computer, and what it costs. A premium, animated page shows every feature that works today, labels everything after V6 "Coming soon", and takes them into the app with one click. Nothing the page does today changes, except the waitlist framing, which is the point.

---

## 0. Contents

1. Scope
2. The no-break rules of this release
3. Decisions (D-316 to D-335)
4. What Offcut does today (the feature inventory)
5. What is coming (the "Coming soon" inventory)
6. The page, section by section
7. The visual system
8. Motion: GSAP and Motion
9. The clips: slots, files, posters, playback
10. Engineering: files, dependencies, budget, lint, security policy
11. The landing page's logic, kept line by line
12. Tests
13. The Dodo fix: the page stops reading as a waitlist
14. Lower captions (DC-13)
15. Part C: the other pages on the dark system
16. Build order
17. Risks
18. Founder decisions, with recommendations
19. Exit checklist
20. Copy-back to the specs

---

## 1. Scope

### 1.1 In this release

| Part | What exists at the end |
|---|---|
| A. Landing page | A new `/`: a dark, premium, animated single page with 12 sections. It names every feature that works today and lists every post-V6 feature as "Coming soon". The drop zone, sample button, capability line, privacy link, pricing, update form and footer are all kept |
| A. Clips | Up to seven silent, looping clips made with Remotion by the founder (prompts already given), served from the asset host. Poster frames are bundled with the app. The hand-made demo (`media/demo.81cafa494d9c2b68.mp4`) is no longer used anywhere |
| A. Dodo fix | The page says the app is open: "Start free" goes to `/app`. No "not open yet" sentence remains (§13) |
| B. Lower captions | The Bottom position moves down, under a new compatibility row, DC-13, with regenerated goldens (§14) **(founder: the option)** |
| C. Other pages | Settings, the legal pages, account, sign-in callback and unsupported move onto the same dark tokens. This is a CSS change only, with the same markup **(founder: yes/no)** (§15) |

### 1.2 Not in this release

| Item | Why |
|---|---|
| Any change to the editor's behaviour, the pipeline, workers, persistence, server, API or analytics | Rule 2.1 |
| New analytics events (for example, CTA clicks) | A new event changes the allowlist, the server enum and the "What leaves your device" table. `landing_view` with `hero_variant: "outcome"` stays the only landing event |
| A/B testing the headline (E-6) | TS §13 keeps the seam; not now |
| Live payments | That is the founding release's decision (D-303), tied to open items 10 and 17 (§13) |
| Cloud transcription | It has its own release next. It appears here only as "Coming soon" |
| Testimonials, user counts, logos of platforms or other companies | Nothing true to show yet. Rule 2.4 |

---

## 2. The no-break rules of this release

These rules are the founder's ("I don't want to break anything, don't change any logic") written as checks.

### 2.1 Files that do not change

The following files stay byte-for-byte unchanged in Parts A and C. Part B touches only the files named in §14.

| Area | Files |
|---|---|
| Entry and routing | `web/src/main.tsx`, `App.tsx`, `routes.tsx` |
| Behaviour layers | everything under `usecases/`, `state/`, `persistence/`, `workers/`, `wasm/`, `models/`, `platform/`, `analytics/`, `entitlement/` |
| Network | `net/http.ts`, `net/api-client.ts`, `config/env.ts`, `config/allowlist-hosts.ts`, `config/pricing.ts`, `web/vercel.json` (no new host, same security policy) |
| Components the landing page uses | `ui/components/DropZone.tsx`, `NotifyMeForm.tsx`, `CapabilityGate.tsx`, `WhatLeavesTable.tsx` |
| The editor | `ui/layout/**`, `ui/primitives/**`, every editor component, `editor.module.css`, `panel.module.css`, `preview.module.css`, `primitives.module.css`, `controls.module.css` |
| Server and crates | `server/**`, `crates/**` (Part B excepted), `verify/**` |

**Proof.** `git diff --stat main...design-build` lists only the files of §10.1, plus Part B's files in Part B's commit.

### 2.2 What the landing page does today, and still does afterwards

§11 lists every behaviour of today's `LandingPage.tsx` (133 lines) and where it lives afterwards. Each behaviour moves verbatim. The navigation effect stays in `LandingPage.tsx` itself.

### 2.3 Tests

All 18 cases of `landing.spec.ts` keep passing, except the cases §12.4 declares. Every other suite runs unchanged: `signin`, `checkout`, the media suites (`drop-zone` on `/app`), Vitest and Rust (Part A touches no Rust). **Test counts only rise** (TS §41.1).

### 2.4 Every claim on the page is true on the day it ships

- Features that work today are stated plainly.
- Post-V6 features carry a "Coming soon" label, with no dates.
- Nothing names TikTok, Instagram, YouTube, CapCut or any other company.
- There are no invented numbers, testimonials or user counts.
- The privacy wording stays factual (TS §25.1 P-12): "Your video stays on your computer." Never "100% private" or legal terms.

### 2.5 Content is never hidden

Content that a person or a test needs is never set invisible while waiting for an animation. That covers the headline, facts, capability line, CTAs, drop zone, form, pricing, roadmap text and footer. Their entrance animations move them (transform) but do not fade them from 0. Only decoration may start at opacity 0.

If GSAP fails to load, or the visitor asks for reduced motion, the page shows the final state at once.

---

## 3. Decisions

**D-316. The landing page moves onto the editor's dark system.**

- Its root carries `data-theme="dark"`, so it reads the tokens of `tokens.css` (`--bg #0e0e11`, `--brand #8b5cf6`, Inter UI and so on).
- Landing-only tokens live in the landing's own CSS module.
- `tokens.css` gains one global rule only: `:root:has([data-theme="dark"]) body { background: #0e0e11 }`. It stops the cream body showing on overscroll; it is CSS only, and the editor benefits too.
- Today's comment in `tokens.css` ("the landing page uses system fonts… never the landing page") is rewritten to match.

**D-317. One concept: "the page is a short".** The page is designed as a 60-second vertical short. This is unique to Offcut because it uses the product's own look rather than a borrowed style:

- **A captioned headline.** The H1 plays like an Offcut Clean caption: each word lights up in turn under the yellow pill (`#FFD60A`, ink `#111114`), which slides from word to word. It rests on the key word.
- **Timecode eyebrows.** Each section's small label is a timecode in JetBrains Mono, for example `00:07  WHAT IT DOES` and `00:21  HOW IT WORKS`. The last section is `01:00  YOUR TURN`.
- **The progress bar.** A 2 px bar across the top of the window fills as the page scrolls, like a short's progress bar (GSAP ScrollTrigger, scrubbed).
- **The playhead.** On wide screens, a thin violet vertical line runs down the left gutter, and a playhead dot travels it with the scroll. Each section's timecode lights up when the playhead reaches it.

**D-318. GSAP and Motion each have a role, and never animate the same element.**

- **Motion** (`motion`, imported from `motion/react`, MIT) is React-side. It handles entrances on view, staggered grids, hover and tap micro-interactions, and the floating chips around the hero phone. It is used through `LazyMotion` with `domAnimation` loaded synchronously and the light `m` component.
- **GSAP** (`gsap` with `ScrollTrigger`) is timeline-side. It handles the captioned headline, the progress bar, the playhead, the pinned "How it works" sequence, the marquee and the hero's pointer parallax. It is loaded by `import()` once the page has rendered, so its chunk never blocks the first paint and never reaches the editor's route.
- Details are in §8.

**D-319. Motion and GSAP are dependencies of the landing page only.**

- An ESLint `no-restricted-imports` rule refuses `gsap`, `gsap/*`, `motion` and `motion/*` in every file outside `web/src/ui/landing/**`.
- **TDR-26** (TS §38) records why: two libraries for a page whose job is to sell, kept out of the product.

**D-320. The app-shell budget rises from 400 kB to 500 kB gzip.** This is the founder's yes of 2026-11-04 ("increase as much as needed").

- **Today.** 351.6 kB.
- **Expected** **(assumption)**: about 425-440 kB. That is GSAP core with ScrollTrigger at about 45 kB, Motion's `m` with `domAnimation` at about 20 kB, and the landing code and CSS at about 15 kB.
- **What gets recorded.** The real number is measured by `check-hosts.mjs` and written in DCL.
- **Spec change.** TS §39.3 item 24 changes from 400 to 500.
- **The editor's route.** It downloads none of the GSAP chunk (§12.3, case L-8).

**D-321. The landing copy gets its own file: `web/src/copy/landing.ts`.**

- `messages.ts` is at 679 of 700 lines.
- `landing.ts` exports `landingCopy` and `pricingCopy`, and `messages.ts` keeps the keys `messages.landing` and `messages.pricing` by importing them. Every caller and test keeps working unchanged.
- INV-15 and TS §5 now read "`copy/messages.ts` and the files it imports".
- The digit rule of TS §11.3 applies to both files: numbers come from `LIMITS` and `PRICES_USD`.

**D-322. The landing page is made of sections in a new folder, `web/src/ui/landing/`.**

- `LandingPage.tsx` stays in `ui/pages/` and stays the route's element, with the same props.
- It keeps its navigation effect and composes the sections.
- The folder holds one component per section, plus the clip player and the motion helpers (§10.1).
- ESLint's `ui` layer already covers the folder.

**D-323. The clips are served from the asset host; their posters are bundled with the app.**

- **Videos.** They are uploaded with `scripts/upload-assets.sh media`, which content-hashes the names, and sit under `media/` on R2. The security policy's `media-src` already allows them.
- **Posters.** They are WebP files in `web/src/ui/assets/landing/`, emitted hashed under `/assets/`, which `img-src 'self'` allows.
- **Paths.** They live in one constant, `LANDING_CLIPS`, in `net/asset-fetch.ts`. The ESLint exception that lets `ui` read `assetUrl` and `DEMO_VIDEO_PATH` from that file gains one name, `LANDING_CLIPS`.

**D-324. `DEMO_VIDEO_PATH` points to the hero clip.**

- The unsupported page keeps its markup and shows the new clip instead of the hand-made demo.
- `messages.landing.demoCaption`, which only that page reads now, gets new words: "What Offcut makes: a raw take in, a finished short out."
- The constant keeps its name, so the ESLint exception and the unsupported page do not change.

**D-325. A slot without a clip shows a still, never a broken video.**

- `LoopVideo` takes `clip?: LandingClip`. With none, it renders the section's CSS illustration or its poster.
- So the page can ship before all seven clips exist, and a missing upload never shows an empty black box.

**D-326. Clips are decorative, silent and polite.**

- Each `<video>` is `muted`, `playsInline`, `loop`, `crossOrigin="anonymous"`, `disablePictureInPicture` and `aria-hidden="true"`.
- **Text alternative.** The section's text says the same thing, and a visually hidden line describes the clip.
- **Playback.** The clip plays only while at least 25% of it is on screen (`IntersectionObserver`), and pauses when the tab is hidden.
- **Reduced motion or Save-Data.** It does not autoplay. The poster shows with a "Play the clip" button.
- **Loading.** Only the hero preloads (`metadata`); the others are `preload="none"` until they come near the viewport.

**D-327. The hero's CTAs are "Start free" (a link to `/app`) and "See how it works" (an in-page link to `#how`).**

- "Try a sample" stays only on the drop zone's button, so its accessible name exists once (§12.1).
- "Start free" carries no new logic. It is a React Router `<Link>`. `/app`'s `CapabilityGate` already decides between the editor and the unsupported page.

**D-328. The waitlist becomes an update list, with the same form and wire.**

- `NotifyMeForm wanted={["launch"]}` is kept unchanged.
- The heading and label change from "Offcut is not open yet…" and "Email me when Offcut opens" to update-list copy (§6, section 10).
- The stored value stays `launch`. Its meaning widens from "the launch" to "news of what opens next". This is recorded in DCL, and needs no server change.

**D-329. Pricing shows the plans, a way in, and no way to buy.**

- The Free card has a "Start free" link to `/app`.
- The Creator card shows its price and features, plus the line "Creator goes on sale soon."
- **(founder, §13)** This holds until the founding release turns on live payments. With live mode on, the line becomes "Upgrade inside Offcut, any time."
- **DECLARED CHANGE** to the pricing test (§12.4).

**D-330. One accent typeface: JetBrains Mono 700, for timecodes, chips and figures only.**

- It is the caption style "Mono"'s own font, already embedded in the scene (`crates/offcut-scene/assets/fonts/JetBrainsMono-Variable.ttf`, OFL).
- It is subset to Latin, digits and `:·→×` as `web/src/ui/assets/jetbrains-mono-latin.woff2` (about 20-25 kB **(assumption)**), with `font-display: swap`, and listed in `web/LICENSES.md`.
- Everything else is Inter UI, already shipped.

**D-331. The page has no third-party anything.**

- No CDN, no font host, no embed, no tracking pixel.
- Everything is served from the app's own origin or the asset host, as today.
- `check-hosts.mjs` stays the proof. A URL literal inside GSAP or Motion's bundle is either removed by the build or added to its tolerated list with a reason, like React's (§10.5).

**D-332. Lower captions are a separate commit, under DC-13, after the founder picks from rendered stills (§14).** No caption, golden or snapshot changes in Part A's commits.

**D-333. Section copy is drafted in this file and finalised in `copy/landing.ts`.** The founder reads the copy at design review 3 (P158), as with the legal pages.

**D-334. Test strings that must stay unique on the page.**

- Playwright's `getByText` and `getByRole(…, { name })` match substrings and fail when two elements match.
- So the strings listed in §12.1 appear exactly once on `/`. In particular, the privacy band does **not** repeat "Your video stays on your computer.", and the footer does not repeat the "What leaves your device" link.

**D-335. The open-graph image is the first thing a shared link shows.**

- `web/index.html` gains a title, a meta description and `og:`/`twitter:` tags, pointing to `/og.png` in `web/public/` (1200 × 630, made from the hero clip's poster).
- These are tags only, and a crawler reads them, not the page's scripts. **(founder: yes/no; recommended yes, since creators share links)**

---

## 4. What Offcut does today (the feature inventory)

Every row below works on the live site at `v6`, and each must be named on the page. Numbers come from `LIMITS` (`gen/domain.ts`) and `PRICES_USD` (`config/pricing.ts`), never typed into copy.

| # | Feature | Source of truth | Where on the page |
|---|---|---|---|
| F1 | Drop a raw talking-head clip: MP4 or MOV, H.264 and AAC, up to `MAX_CLIP_DURATION` (90 s), English, one speaker, up to `MAX_LONG_SIDE` (1920 px) on the long side and `MAX_INPUT_FPS` (60 fps), up to `MAX_FILE_SIZE` (500 MB) | PS §9.4, `LIMITS` | Drop section, limits line |
| F2 | Try a sample clip without a file of your own | `DropZone` | Drop section (its own button) |
| F3 | Transcription on your computer, word by word: a speech model runs in the browser, downloaded once on first use and checked | PS §9.5, `model-manifest.json` | Features: "Transcribed on your computer" |
| F4 | Voice cleanup (noise reduced, high-pass, compression, limiter) and loudness set for phones, with the duration unchanged | PS §9.5 | Features: "A cleaner voice" |
| F5 | Animated word-by-word captions in three styles, Clean, Bold and Mono, plus "Highlight numbers" | `messages.style` | Features: captions (with the caption-styles clip) |
| F6 | Visuals from what you say: Big number, List, Before → after, Emphasis. Each can be turned off or jumped to | `messages.visuals.kind` | Features: visuals (with the visuals clip) |
| F7 | 9:16 framing: a portrait clip passes through; a landscape clip is cropped, and you choose the crop | `FramingPanel` | Features: framing |
| F8 | Caption position: Top, Middle or Bottom, or drag the captions | `FramingPanel` | Features: framing |
| F9 | Live preview with sound: play, seek, frame by frame | `PreviewPlayer` | Features: editor |
| F10 | Fix any word in a click, change all copies of a word at once, hide a word, restore the original; the text follows playback | `messages.captions` | Features: editor |
| F11 | Undo and redo, and keyboard shortcuts | `messages.shortcuts` | Features: editor |
| F12 | Cancel any job | `messages.cancel` | Not marketed; it is a safety feature, not a selling point |
| F13 | Export an MP4: 1080 × 1920 on Creator; 720 × 1280 with a small watermark on Free; rendered on your computer, then downloaded | PS §9.4 | Features: export; Pricing |
| F14 | "What we changed": a summary after the export (voice cleaned, words emphasized, visual moments) | `messages.export.summary` | Features: export |
| F15 | Sign in with an emailed link, with no password | `SignInDialog` | Features: account |
| F16 | Plans: Free (`FREE_EXPORTS_PER_MONTH`, 3 a month, 720p, watermark); Creator (`$15/month or $144/year`, unlimited, 1080p, no watermark) | PS §11, `PRICES_USD` | Pricing |
| F17 | Account page: your plan, manage subscription, export your data, delete your account | `AccountPage` | Features: account |
| F18 | Privacy: the video is not uploaded; a "What leaves your device" page lists every anonymous event; no third-party trackers | PS §17, `/settings` | Hero facts; privacy band |
| F19 | Works in Chrome or Edge on Windows and Mac (desktop) | PS §9.3 | Hero facts; capability line |

**Not claimed.** These do not exist yet, or are not features:

- Reopening a clip after a reload (V7).
- Mobile, Safari, Firefox and Linux (PS §9.2).
- "Saved on this device". It is true of edits, but restore is V7, so the page says nothing that implies "come back later and continue".

---

## 5. What is coming (the "Coming soon" inventory)

Every row carries the chip "Coming soon". There are no dates and no order beyond the grouping. A footnote under the grid reads: "In development. What ships, and when, may change."

| # | Release | Title (draft) | One line (draft) |
|---|---|---|---|
| N1 | Cloud transcription (next) | Faster transcription, your choice | Much faster captions for long recordings, in the cloud if you choose it. On-device stays available |
| N2 | V6.1 | Long videos | Up to 20 minutes and 4K, with transcription that carries on after a closed tab |
| N3 | V6.4 | One long video, many shorts | Offcut picks 5 to 10 shorts from a long video, each with the reason it was picked. Export the ones you keep |
| N4 | V6.2 | An edit plan from your words | Emphasis, emoji, a hook title, a post caption and hashtags, planned from your transcript |
| N5 | V6.3 | Pauses trimmed | Dead air shortened, never your words. Every cut is shown and can be restored |
| N6 | V6.5 | Framing that follows you | A speaker who moves stays in frame |
| N7 | V6.6 | The finished look | Cards for quotes, comparisons, steps and checklists; a hook title, progress bar and end card; caption presets and shape transitions |
| N8 | V6.7 | Music | Your own track, lowered automatically under your voice, and sound cues |
| N9 | V6.8 | B-roll and layouts | Your own B-roll placed for you, transitions at joins, screen-and-face layouts |
| N10 | V6.9 | Edit by asking | Ask for a change in words. No timeline |
| N11 | V6.10 | Brand kit and memory | Your logo, colours and fonts on every short. Offcut remembers what you prefer, and works through your back catalogue |
| N12 | V6.11 | Text behind you, new backgrounds | Text behind the speaker and background changes, released only if they meet our quality bar |

**Grouping.** N1 and N2 sit under "Next". N3 to N12 sit under "On the way".

**Privacy wording.** N1 and N4 send something off the device, so their lines say so plainly ("in the cloud if you choose it", "from your transcript"). The hero's privacy line is today's fact and stays true while N1 is opt-in. If the cloud-transcription release makes cloud the default, the hero line changes in that release, not here.

**Not listed.** V7-V10 (reliability, privacy proof, speed, launch) improve what exists rather than add a feature. Safari, Firefox and mobile are P1 with no release; the unsupported page already offers to write when they come.

---

## 6. The page, section by section

Desktop is 1280-1920 px wide, the design reviews' widths. The page also works down to 360 px, because phone visitors see it even though the app refuses phones.

Copy marked *draft* goes into `copy/landing.ts`, and the founder reads it (D-333). Clip slots are named as in the Remotion prompts.

| # | Section (id) | Timecode eyebrow | Content | Clip slot | Motion |
|---|---|---|---|---|---|
| 0 | Top bar (`nav`) | none | The "Offcut" wordmark (Inter 800 with a small yellow pill on the "O" **(draft)**); links Features · How it works · Pricing · Coming soon (anchors); a "Start free" button. Sticky, glass (`backdrop-filter: blur(12px)`, `rgb(14 14 17 / 0.72)`), hairline border once scrolled. Links collapse below 720 px, and "Start free" stays | none | Motion: border fades in after 8 px of scroll. GSAP: the 2 px progress bar under it (D-317) |
| 1 | Hero (`top`) | `00:00  RAW TAKE IN` | **H1** *draft*: "Say it once. Post it finished." Subhead *draft*: "Drop in a raw talking-head clip. Offcut cleans your voice, writes word-by-word captions and animates the numbers and lists you say, then exports a vertical short ready to post." CTAs: **Start free** (primary) · **See how it works**. Below them, the capability pill (the existing `role="status"`, `data-testid="capability-line"` line, with a green or grey dot), then the two facts: `supportedBrowsers` and `privacyLine` with the `whatLeavesLink`. Right column: the hero clip in a CSS phone frame with three floating chips (*draft*: "Voice cleaned", "Captions, word by word", "Numbers counted up") | `hero` (9:16) | GSAP: the captioned H1 (once, about 2.4 s, resting on "finished"), pointer parallax on the phone (max 6°). Motion: chips float in with a stagger, then drift slowly. CSS: two violet radial glows and a faint grid, masked |
| 2 | Marquee | none | One honest line of what it does, looping: "Word-by-word captions · Voice cleanup · Big numbers · Lists · Before → after · Emphasis · 9:16 framing · 1080p export · Runs in your browser" *draft*. `aria-hidden`; the same facts are in section 4 | none | GSAP: infinite horizontal loop, slowing on hover. Static under reduced motion |
| 3 | Before and after (`before-after`) | `00:04  BEFORE → AFTER` | H2 *draft*: "Raw take in. Finished short out." Three steps under the clip: "Transcribed on your computer" · "Voice cleaned" · "Captions and visuals added" *draft* | `before-after` (16:9) | Motion: the steps light up in turn on view |
| 4 | Features (`features`) | `00:07  WHAT IT DOES` | H2 *draft*: "Everything a talking-head short needs. Nothing it doesn't." A bento grid of 8 cards covering F3-F11 and F13-F17 (rows below this table) | `caption-styles` (9:16) in the Captions card; `visuals` (1:1) in the Visuals card | Motion: cards rise 16 px in a stagger of 60 ms; hover lifts 2 px with a spotlight that follows the cursor (a CSS variable set through `style.setProperty`) |
| 5 | How it works (`how`) | `00:21  HOW IT WORKS` | Three steps: **Drop** ("Drop a clip, or try the sample."), **Review** ("Fix a word, pick a style, move the captions."), **Export** ("Download an MP4 ready to post.") *draft* | `how-it-works` (16:9) | GSAP: on screens 1024 px and wider, the section pins for about 1.5 viewports while the steps advance with the scroll and the clip stays in view. Below 1024 px, or with reduced motion, there is no pin and the steps stack |
| 6 | Try it (`try`) | `00:30  YOUR CLIP` | H2 *draft*: "Try it now." The limits line *draft*, made from `LIMITS`: "Up to 90 seconds, in English, one speaker. MP4 or MOV." Then the **unchanged `<DropZone />`** (its prompt, its "Try a sample" button, its status line) in a glowing frame | none | Motion: the frame's border glows on drag-over (CSS only, through `:has([data-testid="drop-zone"]:hover)`, so DropZone is not touched) |
| 7 | Privacy (`privacy`) | `00:38  ON YOUR COMPUTER` | H2 *draft*: "Not uploaded. Made in your browser." Three facts *draft*: "The video, its sound and its words stay in this browser." · "No third-party trackers." · "Every anonymous event we count is listed." The link to `/settings` is **not** repeated here (D-334); a second link named "See the full list" is allowed, because its name differs | `privacy` (16:9) | Motion: the facts fade and rise on view (decorative check icons only) |
| 8 | Pricing (`pricing`, `data-testid="pricing"`) | `00:44  PRICING` | H2 `pricing.heading`. Two cards. **Free**: `$0`, its features from `LIMITS`, plus "Every feature above"; a **Start free** link. **Creator** (a violet border, "For regular posting" *draft*): `$15/month or $144/year`, its features; the line from D-329. Under both, the `merchant` line | none | Motion: the cards rise; Creator's border has a slow violet sweep (CSS `@property` angle animation) |
| 9 | Coming soon (`soon`) | `00:50  COMING SOON` | H2 *draft*: "Where Offcut is going." The "Next" group (N1-N2), then "On the way" (N3-N12), each a card with the "Coming soon" chip, then the footnote | `coming-soon` (16:9) above the grid | Motion: staggered cards; the chips' dot pulses (CSS) |
| 10 | Updates | `00:56  STAY CLOSE` | Heading *draft*: "Hear when the next features open." The **unchanged `<NotifyMeForm wanted={["launch"]} />`**, with its label *draft* "Email me when new features open" | none | none (a form does not move) |
| 11 | Final call | `01:00  YOUR TURN` | Big line *draft*: "Your next short is one take away." **Start free** | none | GSAP: the line's last word gets the yellow pill when it scrolls in, an echo of the hero |
| 12 | Footer (`data-testid="site-footer"`) | none | Wordmark and one line *draft*: "Speech-driven vertical shorts, made in your browser." The `LEGAL_DOCS` links (Terms, Privacy, Refunds) and **Contact** (mailto), unchanged. A link "Settings" to `/settings` (a name other than `whatLeavesLink`, D-334) | none | none |

**The feature cards of section 4.** Card sizes are on a 3-column grid.

| Card | Size | Title (draft) | Body (draft) | Illustration |
|---|---|---|---|---|
| Captions | 1 × 2 | Captions that follow your voice | "Word by word, timed to your speech. Three styles: Clean, Bold and Mono. Numbers can take the accent colour." | `caption-styles` clip, with three style chips under it |
| Visuals | 2 × 1 | Visuals for what you say | "Say a number, a list or a before-and-after, and Offcut animates it. Turn any of them off." With four chips: Big number · List · Before → after · Emphasis | `visuals` clip |
| Voice | 1 × 1 | A cleaner voice, same length | "Noise reduced, levels evened out and loudness set for phones. Nothing is cut, so your timing stays yours." | CSS waveform: a noisy one turning clean (CSS keyframes) |
| Framing | 1 × 1 | Vertical, framed your way | "Portrait clips pass straight through. Landscape clips are cropped to 9:16, and you pick the crop. Captions go top, middle or bottom, or wherever you drag them." | CSS: a 16:9 frame with a 9:16 window sliding across it |
| Editor | 1 × 1 | Fix any word in a click | "Correct a word, change every copy of it at once, or hide it. Live preview with sound, undo and redo, and keyboard shortcuts." | HTML mock: a caption line where one word is replaced (Motion) |
| Transcription | 1 × 1 | Transcribed on your computer | "A speech model runs in your browser. It downloads once, the first time." | Mono text: timestamps appearing word by word |
| Export | 1 × 1 | Ready to post | "An MP4 in 1080p on Creator, or 720p with a small watermark on Free, rendered on your computer. Then a short summary of what we changed." | Two resolution badges and an MP4 file chip |
| Account | 1 × 1 | Sign in with a link | "No password: we email you a link. See your plan, take your data, or delete your account at any time." | A mail-link chip |

Resolutions are written as the pricing copy already writes them ("720p", "1080p"). Every other number (90 seconds, 3 exports, prices) is interpolated from `LIMITS` and `PRICES_USD`, never typed into a sentence (TS §11.3).

---

## 7. The visual system

### 7.1 Tokens

The landing page uses the `[data-theme="dark"]` tokens of `tokens.css`. It adds these, in `landing.module.css`'s `.landing` root:

```css
--lp-max: 1200px;            /* content width */
--lp-gutter: clamp(20px, 4vw, 48px);
--lp-section: clamp(96px, 12vw, 160px);   /* vertical rhythm between sections */
--lp-h1: clamp(44px, 7vw, 96px);
--lp-h2: clamp(32px, 4.5vw, 56px);
--lp-lead: clamp(17px, 1.6vw, 20px);
--lp-radius-lg: 24px;        /* bento cards, frames */
--lp-pill: #ffd60a;          /* the caption highlight (theme.rs Clean) */
--lp-pill-ink: #111114;
--lp-mono-green: #3ddc84;    /* "on your computer", status dot */
--lp-glow: rgb(139 92 246 / 0.18);
--lp-hairline: rgb(255 255 255 / 0.06);
--font-accent: "JetBrains Mono UI", ui-monospace, monospace;
```

### 7.2 Type

| Role | Face | Size | Weight | Tracking |
|---|---|---|---|---|
| H1 | Inter UI | `--lp-h1` | 800 | -0.035em, line-height 1.02 |
| H2 | Inter UI | `--lp-h2` | 750 | -0.03em, 1.08 |
| Card title | Inter UI | 20-22 px | 650 | -0.01em |
| Body, lead | Inter UI | 15-16 px / `--lp-lead` | 400 | 0, 1.6 |
| Eyebrow, chip, figures | JetBrains Mono UI | 12-13 px | 700 | 0.08em, uppercase |

### 7.3 Colour use

- **Background.** `--bg`, with surfaces `--surface-1` and `--surface-2` for cards.
- **Violet (`--brand`).** For interactive elements and the playhead.
- **Yellow (`--lp-pill`).** Only for the caption pill: the H1, the final call and the Captions card. It is the signature, so it stays rare.
- **Green.** Only for "on your computer" and the capability dot.
- **Contrast.** All text is at least 4.5:1 on its background. `--text-2` `#a3a3ad` on `#0e0e11` is about 8:1. The violet button uses `--brand-hover` under white text for AA, as P108 did.

### 7.4 Components of the page

The following live in `ui/landing/` and are styled by `landing.module.css` and `landing-sections.module.css`:

- `PhoneFrame`, in CSS: radius 44 px, a 10 px `--surface-2` bezel, an inner highlight and a soft shadow.
- `Chip`: 28 px tall, `--surface-2`, a hairline border, mono text and an optional dot.
- `SectionHead`: the timecode eyebrow, H2 and lead.
- `BentoCard`.
- `PlanCard`.
- `SoonCard`.
- `CtaLink`: primary and secondary.

The editor's primitives are not reused, so that the editor stays untouched (§2.1).

### 7.5 Layout

The content column is 1200 px wide. The hero is a 7/5 split, stacked below 960 px. The bento grid has 3 columns at 1024 px and up, 2 at 640-1023 px and 1 below. Sections are separated by `--lp-section`. No section is wider than the viewport, so the page never scrolls sideways.

---

## 8. Motion: GSAP and Motion

### 8.1 Rules

1. **Reduced motion.**
   - Motion is wrapped in `<MotionConfig reducedMotion="user">`.
   - GSAP registers its timelines inside `gsap.matchMedia()` under `(prefers-reduced-motion: no-preference)` only, so with reduced motion nothing is registered.
   - CSS keyframes sit under `@media (prefers-reduced-motion: no-preference)`.
   - Under reduced motion the page is static and complete.
2. **Content is never hidden** (rule 2.5). Motion's `initial` on content uses `y` only; `opacity: 0` is allowed on decoration only. GSAP uses `from()` only after its chunk has loaded, so it never leaves an element hidden if the chunk fails.
3. **One owner per element.** An element animated by GSAP is never an `m.*` element, and the reverse.
4. **Transform and opacity only.** Never animate layout properties (`width`, `top`, `height`). Never use Motion's `layout` prop or `AnimatePresence mode="popLayout"`: the latter inserts a `<style>` element, which `style-src 'self'` would block.
5. **No `style=` in JSX.** Motion values go through `initial`, `animate` and `whileInView` props. GSAP and the spotlight go through refs (`gsap.to(ref.current, …)`, `el.style.setProperty("--x", …)`). Both write through the CSS object model, which the security policy allows. Inline style attributes would not be allowed, and none are written.
6. **Clean-up.**
   - Every GSAP timeline lives in a `gsap.context()` that is reverted on unmount.
   - ScrollTriggers are killed on unmount.
   - Leaving `/` for `/app` (after a drop) leaves no ticker, observer or pin behind.
   - Case L-9 checks this.
7. **Budget.** No long task over 50 ms on the main thread while scrolling (DevTools performance panel, D1). Animations run off scroll through ScrollTrigger's own `scrub`, never a `scroll` listener of ours.

### 8.2 The animations

| # | What | Library | Detail |
|---|---|---|---|
| M1 | Captioned H1 | GSAP | The H1's words are `<span>`s with real spaces between them (the accessible name stays the sentence). A pill `<span aria-hidden>` sits behind the line. A timeline moves the pill to each word (x and width through `getBoundingClientRect`), 0.28 s per word with the brand ease, and the word's colour switches to the pill ink. It plays once and ends on the key word. A resize re-measures. Without GSAP: no pill and white text |
| M2 | Progress bar | GSAP ScrollTrigger | `scaleX` 0 → 1 across the whole page, `scrub: 0.3` |
| M3 | Playhead | GSAP ScrollTrigger | Only at 1280 px and wider. The dot's `y` follows the scroll; each section's eyebrow toggles a class `is-live` (violet) at `top center` |
| M4 | Hero parallax | GSAP `quickTo` | The phone rotates in X and Y by up to 6° from the pointer; it is off on touch devices |
| M5 | Hero chips | Motion | They enter in a stagger of 0.12 s, then drift ±6 px forever (`repeatType: "mirror"`, 6-8 s) |
| M6 | Marquee | GSAP | Two copies of the line, `xPercent: -50` repeating; `timeScale` eases to 0.3 on hover |
| M7 | Section entrances | Motion | `whileInView` with `viewport: { once: true, amount: 0.3 }`, `y: 16 → 0`, 0.6 s, stagger 0.06 s |
| M8 | Bento spotlight | pointer and CSS | A radial gradient at `--x`/`--y` that follows the pointer; the variables are set with `style.setProperty` |
| M9 | How-it-works pin | GSAP ScrollTrigger | `pin: true`, `end: "+=150%"`, with the steps' active state and the clip's caption chips advanced by progress. Only at 1024 px and wider (`matchMedia`) |
| M10 | Final call pill | GSAP | M1's pill on the last word, triggered once at 60% in view |
| M11 | Waveform, crop window, chip dot pulse, Creator border sweep | CSS keyframes | These need no library |

---

## 9. The clips: slots, files, posters, playback

### 9.1 The slots

The founder makes the clips with the Remotion prompts of 2026-11-04, kept in `docs/design/remotion-prompts.md`.

| Slot | Composition | Web file | Duration (loop) | Size cap | Required |
|---|---|---|---|---|---|
| `hero` | hero | 720 × 1280 | 10 s | 2.5 MB | Yes |
| `beforeAfter` | before-after | 1280 × 720 | 12 s | 3 MB | Yes |
| `captionStyles` | caption-styles | 720 × 1280 | 9 s | 2 MB | Yes |
| `visuals` | visuals | 960 × 960 | 12 s | 2.5 MB | Yes |
| `howItWorks` | how-it-works | 1280 × 720 | 12 s | 2.5 MB | No; a CSS illustration otherwise |
| `privacy` | privacy | 1280 × 720 | 6 s | 1.5 MB | No; a CSS illustration otherwise |
| `comingSoon` | coming-soon | 1280 × 720 | 8 s | 1.5 MB | No; nothing otherwise |

**G-1, read on 2026-11-05.** All seven web files exist in `C:\Users\Mayan\Desktop\sh2clips\offcut-remotion\out\`, each with its master and its WebP poster. Results:

- **What passes.** H.264 High, 30 fps, the right dimensions, durations of 10/12/9/12/12/6/8 s, no audio stream, faststart, and sizes of 0.05-0.99 MB, well under every cap.
- **Loops.** Every first frame matches its last.
- **Truth rules.** No other company's name or UI. "Coming soon" stays on screen for the whole coming-soon clip. The how-it-works clip uses real app screenshots.
- **What fails.** The pixel format is `yuvj420p` (full range), not `yuv420p`. P156 re-encodes the seven web files from their masters to `yuv420p`, limited range, BT.709, before the upload (the guide, Phase 6).
- **Footage.** `public/footage/` was never filled, so the clips show a drawn silhouette, not the founder. That is allowed (no stock person, no other company). A later re-render with real footage is a new upload and a new path, with no code change.

### 9.2 Checks before an upload

These run on the founder's `out/` folder and are recorded in DCL:

- `ffprobe`: H.264, `yuv420p`, 30 fps, **no audio stream**, the `moov` atom before `mdat` (faststart), dimensions as above, size under the cap.
- The loop: the first and last frames differ by no more than a dissolve (read by eye on D1).
- The truth rules: every claim on screen is in §4, or carries "Coming soon" and is in §5. No other company's name or logo. The only person on screen is the founder.

### 9.3 Upload and paths

`sh scripts/upload-assets.sh media <files>` prints the hashed paths. They go into:

```ts
// net/asset-fetch.ts
export type LandingClip = { path: string; width: number; height: number };
export const LANDING_CLIPS: Partial<Record<LandingSlot, LandingClip>> = { hero: { path: "media/offcut-hero-720x1280.<hash>.mp4", width: 720, height: 1280 }, … };
export const DEMO_VIDEO_PATH = LANDING_CLIPS.hero?.path ?? "media/demo.81cafa494d9c2b68.mp4";
```

The slot type `LandingSlot` is a union of the seven names. The hand-made demo stays on R2 (nothing is deleted from the asset host), but nothing references it once the hero exists.

### 9.4 Posters

The `.webp` posters (under 80 kB each) go to `web/src/ui/assets/landing/<slot>.webp`. The section imports them, so Vite emits them hashed and immutable, and `img-src 'self'` covers them. The poster gives the `<video>` its box before the clip loads, so the layout does not shift (`width` and `height` attributes from `LandingClip`).

### 9.5 `LoopVideo`

`LoopVideo` is the one component that plays a clip (`ui/landing/LoopVideo.tsx`):

- It renders `<video>` with the attributes of D-326, `poster`, and `src={assetUrl(clip.path)}`.
- It sets `el.muted = true` in a ref effect before any `play()` (React does not reflect `muted` to the attribute).
- An `IntersectionObserver` (`threshold: 0.25`, `rootMargin: "200px"` for loading) calls `play()` and `pause()`. A `play()` rejection is caught and leaves the poster.
- It pauses on `visibilitychange` to hidden.
- Under reduced motion or `navigator.connection?.saveData`, it does not autoplay and shows a "Play the clip" button (the copy key `landing.playClip`), which calls `play()` once.
- With no `clip`, it renders `children` (the section's CSS illustration).

The isolation headers already allow the asset host: today's demo `<video crossOrigin="anonymous">` loads under `Cross-Origin-Embedder-Policy: require-corp`, using R2's CORS for the site. Case L-6 proves it for each clip.

---

## 10. Engineering: files, dependencies, budget, lint, security policy

### 10.1 Files

Every new path is added to TS §5's tree in the same commit, or `check-file-tree.mjs` fails.

| Path | New or changed | What |
|---|---|---|
| `web/src/ui/pages/LandingPage.tsx` | changed | The same props and navigation effect. It renders `<div data-theme="dark" className={styles.landing}>` with the sections inside `<LazyMotion features={domAnimation} strict><MotionConfig reducedMotion="user">`, and keeps `<main>` |
| `web/src/ui/landing/TopBar.tsx` | new | Section 0, with the progress bar element |
| `web/src/ui/landing/Hero.tsx` | new | Section 1, holding `capabilityLine()` moved verbatim from `LandingPage.tsx` (§11) |
| `web/src/ui/landing/CaptionedHeadline.tsx` | new | M1 and M10 |
| `web/src/ui/landing/Marquee.tsx` | new | Section 2 |
| `web/src/ui/landing/BeforeAfter.tsx` | new | Section 3 |
| `web/src/ui/landing/Features.tsx` | new | Section 4 and its 8 cards |
| `web/src/ui/landing/HowItWorks.tsx` | new | Section 5 |
| `web/src/ui/landing/TryIt.tsx` | new | Section 6, holding `<DropZone />` |
| `web/src/ui/landing/Privacy.tsx` | new | Section 7 |
| `web/src/ui/landing/Pricing.tsx` | new | Section 8, from today's pricing markup with the same `data-testid`, heading id and sources |
| `web/src/ui/landing/ComingSoon.tsx` | new | Section 9 |
| `web/src/ui/landing/Updates.tsx` | new | Section 10, holding `<NotifyMeForm wanted={["launch"]} />` |
| `web/src/ui/landing/FinalCall.tsx` | new | Section 11 |
| `web/src/ui/landing/SiteFooter.tsx` | new | Section 12, from today's footer markup |
| `web/src/ui/landing/LoopVideo.tsx` | new | §9.5 |
| `web/src/ui/landing/parts.tsx` | new | `PhoneFrame`, `Chip`, `SectionHead`, `CtaLink`, `BentoCard` |
| `web/src/ui/landing/gsap.ts` | new | `loadGsap()`: a cached `import("gsap")` and `import("gsap/ScrollTrigger")`, `registerPlugin` and `useGsap(scope, setup)`, a hook that runs `setup` inside `gsap.context` and `matchMedia` and reverts on unmount |
| `web/src/ui/styles/landing.module.css` | new | Root, tokens, top bar, hero, shared parts |
| `web/src/ui/styles/landing-sections.module.css` | new | Sections 2-12; splits at 700 lines as needed |
| `web/src/ui/assets/jetbrains-mono-latin.woff2` | new | D-330 |
| `web/src/ui/assets/landing/*.webp` | new | Posters (§9.4) |
| `web/src/copy/landing.ts` | new | D-321 |
| `web/src/copy/messages.ts` | changed | `landing` and `pricing` now come from `landing.ts`; `notifyMe.label` gets new words (D-328) |
| `web/src/net/asset-fetch.ts` | changed | `LandingSlot`, `LandingClip`, `LANDING_CLIPS`; `DEMO_VIDEO_PATH` repointed (D-324) |
| `web/src/ui/styles/tokens.css` | changed | The JetBrains Mono `@font-face`, the `:has()` body rule, and the header comment (D-316) |
| `web/eslint.config.js` | changed | `LANDING_CLIPS` added to the `asset-fetch.ts` specifier list; the `no-restricted-imports` rule of D-319 |
| `web/package.json`, `pnpm-lock.yaml` | changed | `gsap` and `motion`, pinned to exact versions |
| `web/LICENSES.md` | changed | JetBrains Mono (OFL-1.1); a note on GSAP's licence (§10.2) |
| `web/index.html`, `web/public/og.png` | changed / new | D-335 **(founder)** |
| `scripts/check-hosts.mjs` | changed only if needed | Tolerated literals of GSAP and Motion, with reasons (§10.5) |
| `web/tests-e2e/landing.spec.ts` | changed | §12 |
| `docs/technicalspec.md` | changed | §5 tree, INV-15 wording, §39.3 item 24, TDR-26, §41.2 DC-13 (Part B) |

`web/src/ui/pages/UnsupportedPage.tsx` is **unchanged**: it reads `DEMO_VIDEO_PATH` and `messages.landing.demoCaption` as before.

### 10.2 Dependencies

**`motion`** (MIT).

- Use the current stable version at install time, pinned exactly.
- Import only from `motion/react` (`LazyMotion`, `domAnimation`, `m`, `MotionConfig`).

**`gsap`** (current stable 3.x, pinned exactly).

- **Licence.** GSAP has been free for all uses, plugins included, under its own "Standard License" since its sale to Webflow. That licence is not an OSI licence. It forbids using GSAP in a product that competes with Webflow's visual website builder. Offcut is a video tool, so this does not apply.
- The implementer reads the licence text in the installed package before the first commit and quotes its first paragraph in DCL. If the text says anything else, they stop and ask the founder **(founder: accept)**.
- `cargo deny` covers Rust only; there is no JS licence gate to update.

Neither library adds a network request, a CDN, a worker or a `<style>` element when used as §8.1 says.

### 10.3 Bundle

- **Budget.** It becomes 500 kB (D-320).
- **The GSAP chunk.** `loadGsap()`'s dynamic import puts GSAP in its own chunk. Vite's `build.rollupOptions.output.manualChunks` is not needed; the dynamic import is enough.
- **What `check-hosts.mjs` counts.** It counts every `.js`, `.css` and `.html` in `dist`, so the chunk counts towards the budget, as it should.
- **What is recorded in DCL.** The total, the main chunk, and the GSAP chunk.
- **Expected.** The main chunk grows by Motion and the landing code (about 35 kB **(assumption)**).

### 10.4 Lint

- `react/jsx-no-literals` applies to every new `.tsx`: all words come from `copy/`, and punctuation is allowed.
- The boundaries rules are unchanged except for the one specifier (§10.1).
- The new `no-restricted-imports` rule keeps GSAP and Motion inside `ui/landing/` (D-319).
- `check-file-tree.mjs`: no file over 700 lines.

### 10.5 Security policy and hosts

- `web/vercel.json` does not change.
- GSAP and Motion write styles through the CSS object model (`element.style.*`, `setProperty`), which `style-src 'self'` does not govern. Neither injects `<style>` when used as §8.1 says.
- **The proof** is `@smoke` (security policy violations recorded from the first script) and case L-7, which scrolls the whole page and asserts zero violations.
- `check-hosts.mjs` reads the built bundle.
- **If GSAP or Motion leave a URL literal** (a licence link in a banner comment, for example), it is added to `NON_NETWORK_LITERALS` with `exact`, a `chunk` pattern and the reason "a URL in the text of a licence comment; the library makes no request". This needs the same proof as the others: a network capture shows no request to it.

---

## 11. The landing page's logic, kept line by line

| Today in `LandingPage.tsx` (lines) | Afterwards |
|---|---|
| Props `settingsPath`, `appPath`, `legalPath` (30-38) | Same props, same `routes.tsx` call. Passed down to the Hero (settings link), the CTAs (`appPath`), Pricing (`appPath`) and SiteFooter (`legalPath`, `settingsPath`) |
| `capabilityLine(state)` (20-28) | Moved verbatim to `Hero.tsx`, with the same three outcomes and the same copy keys |
| The `<p role="status" data-testid="capability-line">` with `supported`/`muted` classes (68-74) | In the Hero, the same element type, role, test id and text; restyled as a pill (two classes in `landing.module.css`) |
| The clip-arrival navigation effect (43-55) | Stays in `LandingPage.tsx`, character for character |
| `copy.hero` H1 (60) | `CaptionedHeadline` renders an `<h1>` whose text content is `messages.landing.hero` (words in spans, real spaces) |
| `copy.subhead`, `supportedBrowsers`, `privacyLine` + `<Link to={settingsPath}>{whatLeavesLink}</Link>` (61-67) | In the Hero, each once |
| Demo `<figure><video controls …>` (77-86) | **Removed** from `/` (founder, 2026-11-04). Replaced by the `hero` slot. The unsupported page keeps its own (D-324) |
| `<DropZone />` (88) | In `TryIt.tsx`, unchanged, with `variant` defaulting to `landing` |
| Pricing section: `data-testid="pricing"`, `aria-labelledby="pricing-heading"`, plans from `PRICES_USD` and `LIMITS`, `merchant` line (90-116) | In `Pricing.tsx`, with the same test id, heading id and sources; adds the Free card's "Start free" link (D-329) |
| Waitlist section with `<NotifyMeForm wanted={["launch"]} />` (118-121) | In `Updates.tsx`, with the same component and props; a new heading (D-328) |
| Footer `data-testid="site-footer"`, `LEGAL_DOCS` links, mailto (123-130) | In `SiteFooter.tsx`, with the same test id, links, names and href |
| `landing_view` (`start-app.ts`, not this file) | Untouched |

**Restyling DropZone and NotifyMeForm without touching them.** Their classes (`.dropTarget`, `.secondaryButton`, `.notice`, `.form`, `.field`, `.button`) live in `components.module.css` and are shared with `SignInDialog` and the unsupported page. They stay as they are. The landing restyles them from outside, through the wrapper's scope and element or attribute selectors in `landing-sections.module.css`:

```css
.tryFrame [data-testid="drop-zone"] { … }
.tryFrame button { … }
.updates form input { … }
.updates form button { … }
```

The wrapper class plus the selector outranks the single module class, so no `!important` is needed.

---

## 12. Tests

### 12.1 Strings that appear once on `/` (D-334)

Each of these is located by a test and must match exactly one element. Exact names and substrings are both checked by case L-10.

| String | Located as |
|---|---|
| `messages.landing.hero` | heading |
| `messages.landing.supportedBrowsers` | text |
| `messages.landing.privacyLine` | text |
| `messages.landing.whatLeavesLink` | link |
| `messages.dropZone.sampleButton` | button |
| `messages.notifyMe.submit` | button |
| `messages.notifyMe.emailLabel` | label |
| `messages.pricing.heading` | heading inside `pricing` |
| `messages.pricing.notOnSale` (or its replacement) | text inside `pricing` |
| Each `LEGAL_DOCS` link label, and `footerContact` | link inside `site-footer` |

There is one `<form>` on `/`, because `formStatus` is `page.locator("form").getByRole("status")`.

### 12.2 Cases that stay as they are

These 17 cases of `landing.spec.ts` stay as they are:

- J1.
- The settings link.
- `landing_view`.
- `capability_check`.
- The four waitlist cases.
- The dropped file.
- The session start.
- No wait for a session.
- `/app` never blank.
- The hosts of a session.
- `@smoke`.
- Both footer cases.
- The unknown legal document.

### 12.3 New cases

These go in `landing.spec.ts` (non-media; about 30 s added).

| # | Case |
|---|---|
| L-1 | Every feature of §4 marked "on the page" is named. For each feature card title, `messages.landing.features.*.title` is visible |
| L-2 | Every row of §5 is shown with "Coming soon": the count of soon cards equals `landingCopy.soon.items.length`, and each holds the chip |
| L-3 | "Start free" in the hero, top bar, pricing and final call each goes to `/app` (`href` read, plus one click), and `/app` shows the drop zone or the unsupported page (the existing helper) |
| L-4 | No sentence on `/` says the app is not open. The page's text contains none of the old `waitlistHeading`, "not open yet" or "Leave your email and we will tell you when it is" |
| L-5 | Reduced motion (`page.emulateMedia({ reducedMotion: "reduce" })`): every clip is paused with its poster showing and a "Play the clip" button; the H1 has no pill; after 1 s `document.getAnimations()` is empty and no pin spacer exists (the static final state) |
| L-6 | Each clip in `LANDING_CLIPS` loads under the isolation headers: `readyState >= 2` once scrolled into view, with no error event (skipped for slots without a clip) |
| L-7 | Scrolling from top to bottom in 20 steps records zero security policy violations, and `crossOriginIsolated` stays true |
| L-8 | Opening `/app` directly downloads no GSAP: of every `.js` response of the session, none contains the text `ScrollTrigger`. On `/`, after a scroll, one does |
| L-9 | After "Start free" moves the page to `/app` inside the app (no reload), no ScrollTrigger pin spacer (`.pin-spacer`) and no landing root (`[data-landing]`) stays in the DOM, and no `pageerror` was raised |
| L-10 | The strings of §12.1 each match once (`toHaveCount(1)`) |
| L-11 | The `axe(page)` helper of `tests-e2e/helpers/fixtures.ts` (serious and critical violations, D-192) on `/` at 1280 × 800, at the top and after scrolling to the end: an empty list, as `review-edit.spec.ts` case 16 asks of the editor |

### 12.4 DECLARED CHANGES to existing cases

| Case | What changes | What stays | Why |
|---|---|---|---|
| "pricing: the plans show their prices and limits, say they are not on sale, and offer no way to buy" | Renamed to "…and offer no way to buy, only a way in". `notOnSale` is read from its new words (D-329). `getByRole("link")` inside `pricing` has a count of 1, named "Start free", with `href` `/app`, in place of 0 | Every price and feature assertion; **no button** inside `pricing` (count 0); no link to a checkout | The Dodo fix: the page must lead into the app (open item 10) |
| J1 | Only if the founder changes the H1. The case reads `messages.landing.hero`, so its code does not change | Everything | |

No other case changes. If an implementer finds another case failing, they stop and ask (TS §41.4): they do not edit the case.

### 12.5 Screens and review

- `screens.spec.ts` gains a "landing" capture at the two reference widths, plus 390 px for phones.
- The captures go to `~/design-review/` for design review 3, as `~/v4-review-2` did.
- The founder reads them with the creator friend who gave the feedback.

### 12.6 CI

The Linux job already runs `landing.spec.ts`. Nothing else is added; one CI run per batch (memory rule).

---

## 13. The Dodo fix: the page stops reading as a waitlist

Dodo put the product review on hold on 2026-11-04 because the live page said "Offcut is not open yet" and offered no way in (V6 CL open item 10). After this release:

1. No sentence says the app is not open (case L-4). "Start free" appears four times and leads to `/app`, where a visitor can drop a clip, sign in and export on the Free plan. All of that has been live since V6.
2. Pricing shows both plans with prices. Creator's line is the founder's choice:

| Option | Creator's line | What it means | Recommendation |
|---|---|---|---|
| (a) | "Creator goes on sale soon." | Payments stay in test mode until the founding release (D-303), which is true today. In-app offers still lead to Dodo's test mode (open item 17), so the founder decides separately whether to hide them until then | **Recommended**, together with resubmitting to Dodo with the site open. If Dodo then asks for a purchasable plan, go to (b) |
| (b) | "Upgrade inside Offcut, any time." | Dodo's live mode is on and the founding offer is moved forward, which is the founder's decision under D-303 and BP §8A.6 | Only once Dodo approves the product |

3. The legal pages already say "Offcut is in development, and parts of it are not open yet". That stays true and needs no change.

**Found and carried (not fixed here).** Dodo showed an Indian buyer **INR** (1,476.89) while the page names dollars (V6 CL, TE-9 "Found"). The `merchant` line keeps "Prices are in US dollars". Whether Dodo's live mode shows local currency is read in the founding release.

---

## 14. Lower captions (DC-13)

### 14.1 What is true today

- **Where the block sits.** At Bottom (placement 1.0), the caption block is y 1164-1444 of 1920 (`placement.rs`, `CAPTION_BOTTOM = 1304` centre).
- **How lines sit in it.** Lines are **centred** vertically in the 280 lp block (`captions.rs:367`).
- **Where the text lands.** Clean's 72 lp size gives about 87 lp a line **(assumption: ascent plus descent of Inter at 1.21 em)**. So a one-line caption is centred at 1304, 68% of the way down. That is why it "feels like the middle".
- **What is under the block.** The watermark's band (56 lp) sits under the block, and the safe area ends at 1500 (`INSET_BOTTOM 420`).
- **What reads the watermark's place.** The Free watermark is drawn at the safe area's bottom-left. `verify/verify_mp4.py` finds it with a fixed mask (`watermark_mask_720.png`). Moving the watermark therefore changes the verifier's input, which is a protected proof (TS §41.1).

### 14.2 Options

The founder chooses from rendered stills: the reference clip at Bottom, one-line and two-line, in Clean, Bold and Mono, for each option. That is 18 stills in `~/design-review/captions/`.

| Option | Change | One line moves down | Two lines move down | Touches | Risk |
|---|---|---|---|---|---|
| **A. Bottom-hug** (recommended) | Lines align in the block by the placement: `top = block.y + (block.h - H) * p` instead of `* 0.5`, keeping each style's vertical decoration (pill padding, Mono's panel) inside the block | about 96 lp (to about 73% down) | about 53 lp | `captions.rs` (one line and a parameter), goldens and snapshots at non-middle placements | Low. The watermark, safe area, verifier and event zone are unchanged |
| B. A + a smaller watermark gap | A, plus `WATERMARK_BAND` 56 → 48 (the box is about 47 lp) | about 104 lp | about 61 lp | A, plus `placement.rs` | Low. A little gain for a tighter look above the watermark |
| C. A + a lower safe area | A, plus `INSET_BOTTOM` 420 → 340: the watermark follows the safe area down by 80 lp, and the verifier's mask is regenerated (DC-14) | about 176 lp (to about 77% down) | about 133 lp | A, plus `safe_area.rs`, `verify/watermark_mask_720.png`, the event zone and the Full zone | Medium. Captions enter the area where TikTok, Reels and Shorts draw their buttons and description. It changes a protected verifier input |

**Recommendation: A.** It answers "feels like the middle" for the common one-line case. It moves nothing else, and it keeps the verifier independent. C goes further, but it trades into the platforms' UI and changes the verifier. B is A with a small extra gain.

### 14.3 What DC-13 says (for TS §41.2)

> | DC-13 | Design release | At placements other than 0.5, caption lines align in the caption block by the placement (top-aligned at 0, bottom-aligned at 1) instead of centred. Clips saved at Bottom render their captions lower on reopening. Goldens and display-list snapshots of non-middle placements are regenerated; their diffs are only glyph runs', pills' and panels' `y` | The block, event zone, safe area, watermark, `CaptionPlacement` values and their storage; every frame at placement 0.5; the export's 11 checks | `layout_safe_area.rs`, `display_list_snapshots.rs` (diff read), `golden_frames.rs` (regenerated, count recorded), `export-*.spec.ts`, `preview-sync.spec.ts`, `review-edit.spec.ts` |

(Option B or C adds its own lines to this row, and C adds DC-14 for the mask.)

### 14.4 Order

1. Render the stills from a development build with the option behind a local constant (not committed).
2. The founder picks.
3. Write the row in TS §41.2.
4. Make one commit with the code, the regenerated goldens (`UPDATE_GOLDENS=1` as in V4) and the snapshot review.
5. Run the media suites once (memory rule: one full media run where required).

**What is unchanged in the editor.** The preview, drag and Framing tab do not change. The drag still writes the same 0..1 placement, and only where the lines sit in the block moves.

---

## 15. Part C: the other pages on the dark system **(founder: yes/no; recommended yes)**

**What changes.** After Part A, a visitor who clicks Terms, Settings or Account leaves a dark, premium page for V1's cream one. Part C:

- adds `data-theme="dark"` to the root element of `SettingsPage`, `LegalPage`, `AccountPage`, `AuthCallbackPage`, `UnsupportedPage` and `CapabilityGate`'s checking state;
- maps `pages.module.css`'s V1 variables to the dark tokens;
- makes no markup, copy or logic change.

**Tests.** `signin.spec.ts` and `checkout.spec.ts` read text and roles, not colours, so they stay as they are. An axe pass on `/settings` and `/legal/terms` is added (L-12).

**Cost.** About half a day.

---

## 16. Build order

Each prompt makes one commit on `design-build`. The prompts are in `coding-promptsdesign.md` and the commands in `designbuildguide.md`. Prompt numbers continue V6's (P150).

| Prompt | Work | Proof |
|---|---|---|
| Setup | The baseline: `pnpm check`, `pnpm test`, `pnpm e2e`, the app-shell size, and the counts, recorded in DCL. The draft pull request | Counts recorded |
| P151 | Dependencies (`gsap`, `motion`), the GSAP licence read, `copy/landing.ts` with all copy of §4-6, `messages.ts` importing it, the ESLint rules, the TS §5 rows | Tier 1; all 18 landing cases unchanged |
| P152 | Tokens, fonts, `landing.module.css`, `parts.tsx`, `LoopVideo`, `gsap.ts` (the loader only); `LandingPage.tsx` re-composed with TopBar, Hero, TryIt, Pricing (unchanged content), Updates and SiteFooter | The 18 cases unchanged; L-3, L-10 |
| P153 | Sections 2-5 and 7-11 complete (static), Pricing with D-329 (the declared change of §12.4), Updates with D-328 | 17 cases unchanged, plus the pricing case as declared; L-1, L-2, L-4 |
| P154 | Motion: M5, M7, M8 (Motion) and M11 (CSS) | L-5 (reduced motion), L-7 |
| P155 | GSAP: M1, M2, M3, M4, M6, M9, M10 | L-7, L-8, L-9; size recorded; no long task |
| P156 | The clips: re-encode (§9, G-1), upload (with the founder's go), `LANDING_CLIPS`, the posters, `DEMO_VIDEO_PATH` repointed, `og.png` and the meta tags | L-6; `@smoke`; the unsupported page shows the new clip |
| P157 | Part C: the other pages on the dark tokens | L-12; `signin` and `checkout` unchanged |
| P158 | axe (L-11), the `screens.spec.ts` captures of `/`, and the fixes from both; then design review 3 (**founder**) | 0 serious or critical; the founder's "go" in DCL |
| P159 | Part B: the stills, then DC-13's code and goldens | §14.3's proofs; one full media run |
| P160 | Copy-back (§20), the closing entry. The founder merges, the deploy checks follow, Dodo resubmission | Tag `design` (founder) |

**Estimate.** About 5-7 working days. The clips already exist (G-1, §9.1), so nothing waits on the founder before P158's review.

---

## 17. Risks

| Risk | Sign | Answer |
|---|---|---|
| The animation libraries break the security policy | `@smoke` or L-7 sees a violation | §8.1's rules 4 and 5; if a feature needs `<style>`, drop the feature |
| The page is heavy on a slow laptop | A long task over 50 ms while scrolling; LCP over 2.5 s on DevTools' "Fast 4G" **(assumption: targets)** | Fewer pins; the hero poster as the LCP image; clips start later |
| The clips miss the truth rules (a feature not built, a platform's UI) | The §9.2 read | Re-render; the slot shows its still meanwhile (D-325) |
| A clip fails to load under the isolation headers | L-6 | R2's CORS for the site, as for the demo today |
| Copy duplicated across sections makes a test's locator ambiguous | L-10 | D-334's list; rename the duplicate |
| Motion's `initial` hides content when JavaScript fails | Content missing in a no-JS read | Rule 2.5: `y` only on content |
| The caption change reads worse in Mono or Bold | The stills | The option is chosen from stills; "keep as is" is an allowed answer |
| GSAP's licence changes | §10.2's read | Stop and ask; Motion alone can do M1-M10 with more code (about a day) |
| Dodo still refuses | Its answer to the resubmission | Option (b) of §13: live mode earlier, the founder's call |

---

## 18. Founder decisions, with recommendations

**Decided on 2026-11-05.** The founder took every recommendation below ("do what you recommend"):

- **F-1:** the H1 is "Say it once. Post it finished."
- **F-2:** option (a).
- **F-3:** a separate change after this release.
- **F-4:** option A. The stills are still rendered in P159 as a check: if A reads worse in any style, the prompt stops, and "keep as is" stays an allowed answer.
- **F-5:** yes.
- **F-6:** yes.
- **F-7:** yes.
- **F-8:** yes, after the read of §10.2.
- **F-9:** all three optional clips exist, so all seven slots are filled.

| # | Decision | Recommendation |
|---|---|---|
| F-1 | The H1 | "Say it once. Post it finished." (`hero_variant` stays `outcome`) |
| F-2 | Creator's line on the page (§13) | Option (a), "Creator goes on sale soon.", and resubmit to Dodo |
| F-3 | Hide the in-app offers until live mode (open item 17) | Yes, but in its own small change after this release, not inside it (it is logic) |
| F-4 | Caption option (§14) | A, picked from the stills |
| F-5 | Part C, the other pages dark | Yes |
| F-6 | The accent font, JetBrains Mono | Yes |
| F-7 | The open-graph image and meta tags | Yes |
| F-8 | Accept GSAP's licence | Yes, after the read of §10.2 |
| F-9 | Which optional clips (how-it-works, privacy, coming-soon) | All three if time allows; CSS stills otherwise |
| (given) | Raise the app-shell budget | Given on 2026-11-04: 500 kB |

---

## 19. Exit checklist

- [ ] `git diff --stat main...design-build` touches only §10.1's files (plus Part B's and C's).
- [ ] The 17 unchanged cases of `landing.spec.ts` pass unmodified. The pricing case changed only as §12.4 declares.
- [ ] L-1 to L-11 pass, and L-12 passes with Part C.
- [ ] `signin`, `checkout` and the media suites pass (one full media run, for Part B).
- [ ] Rust, Vitest and E2E counts are not below Setup's baseline.
- [ ] App shell is 500 kB or less. The size is recorded, with the GSAP chunk alone.
- [ ] `check-hosts.mjs` is ok: no new host, and every tolerated literal has a reason.
- [ ] axe reports 0 violations on `/`.
- [ ] Every feature of §4 marked "on the page" is on the page. Every row of §5 shows "Coming soon".
- [ ] No sentence says Offcut is not open.
- [ ] Every clip has passed §9.2's checks and plays under the isolation headers. The hand-made demo is referenced nowhere.
- [ ] Reduced motion leaves a complete, static page.
- [ ] DC-13 is in TS §41.2 with the founder's choice.
- [ ] Design review 3 ended in "go".
- [ ] The copy-back is done. The founder merges, the deploy checks pass, and the Dodo resubmission is sent.

---

## 20. Copy-back to the specs

| Spec | What changes |
|---|---|
| TS §5 | The new files of §10.1 |
| TS §13 (the `/` row) | "Hero with the captioned headline, the hero clip (asset host), sections of features and coming soon, `DropZone`, pricing with a way in, the update form" |
| TS §39.3 item 24 | Budget 500 kB, with the reading |
| TS §38 | TDR-26: GSAP and Motion, on the landing page only |
| TS §41.2 | DC-13 (and DC-14 if option C) |
| INV-15 | "`copy/messages.ts` and the files it imports" |
| PS §10 J1 | The landing page's new content; the demo clip replaced by product clips |
| PS §11 | Unchanged prices; the landing's "Start free" |
| BP | A line for the design release between V6 and the cloud-transcription release |
| V6 CL open item 10 | Resubmitted on (date), answer |
