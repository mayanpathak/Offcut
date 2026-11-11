# Offcut - Speech-driven short-form video maker that runs in your browser (Product Spec v1.5)

> **Positioning (launch pitch, v1.4).** Two paths, equal on the landing page, each with its own page. **Make shorts from a long video** (up to 45 minutes) and get 5-10 finished vertical shorts, each built from the best moments, even from different parts of the video, with face-tracked framing that follows whoever is speaking (v1.5), kinetic captions, constant gentle motion, sound effects, automatic colour and light, B-roll and visuals that follow what you said. **Edit a clip** (up to 90 seconds) and get one finished short. The editing all renders on your computer: your video is not uploaded. To write captions fast, the sound is sent for transcription: for a clip when you choose Cloud in the drop zone, which is the default for an account (§12.6), and always for a long video (§12.9). An AI edit plan reads your transcript and decides what goes where; only text and timings are sent to make the plan (§12.8). You edit by clicking or by asking in words; there is no timeline. Flat price, no per-minute credits, because the heavy work runs on your machine (§11). **Not a launch promise:** avatars, phones, languages other than English.

> *Earlier pitch, for reference (v1.2):* a raw talking-head clip up to 90 seconds becomes a finished vertical 1080p short, with an AI edit plan, kinetic captions, cards and a brand kit.

**Companion document:** `technicalspec.md` is the implementation contract. **This document wins any scope, priority, pricing or success-number conflict.** Priority labels (P0/P1/P2), milestone IDs (M0-M4), experiment IDs (E-1 to E-10), hypothesis IDs (H1-H4), assumption IDs (A-1 to A-12) and reason codes (a)-(f) are defined here and used identically in the tech spec.

**Shared labels**

| Label | Meaning |
|---|---|
| P0 / P1 / P2 | Launch-blocking / post-MVP / later |
| Reason codes for every cut | (a) does not test the core hypothesis; (b) disproportionate operational burden or ongoing cost; (c) unnecessary platform, legal or compliance risk; (d) effectively a second product inside the first; (e) users do not need it for the primary workflow; (f) complexity cost (correctness, debugging, maintenance, infra, security, cognitive load) exceeds product value |
| Statement kinds | **VERIFIED FACT** (derived or checked in this document), **ASSUMPTION**, **TARGET**. Anything about competitors or platforms that I found only in third-party 2026 articles and could not check against primary pages is labeled *general-market knowledge, re-verify before launch* |
| M0-M4 | Milestones (dates in §9.9 and §13). These are **not** the same as "Month n" in the growth model |
| "Month 0 / Month 1..." | Calendar growth-model months (§13). Month 0 = build + stabilization; Month 1 = first 30 days after public launch |

**Revision notes**
- v1.4 (founder decisions of 2026-11-08, `docs/v6/v6changelog.md` Entries 21 and 22): **the long path is reshaped before it is built.** Two separate pages: "Make shorts from a long video" and "Edit a clip"; the person chooses, and both open the same editor. One flow on both: the transcript and a prompt describing everything the editor can do go to the AI; the plan is validated and **the editor applies it automatically**, and the person switches off or undoes what they do not want. Long videos **up to 45 minutes**, with no file-size limit: the file is read where it is and never copied whole; only the parts chosen for shorts are kept. **A long video is transcribed in the cloud only;** if that fails, the job pauses with Continue. **B-roll from the person's own clips and from Pixabay stock**, placed by the plan or by hand on a word, with a choice of which part of a clip plays. **Auto zoom** and Submagic-class visual editing; quality is judged against the founder's benchmark after all features work. The build order stays as v1.3 set it. Tooscut stays a study source for the editing engine, never copied.
- v1.3 (founder decisions of 2026-11-02): **Offcut adds a long path.** A video up to 20 minutes (v1.4: 45), with 4K and HEVC input, becomes 5-10 finished shorts, each made of whole sentences chosen by a Director request and optionally stitched from different parts (§12.9). Added: local vision signals and face-tracked framing, B-roll from the user's own asset box, transitions at joins, chat editing, creator memory, a back-catalogue queue, text behind the speaker and background change (gated on quality). A paid AI provider is allowed with a spending cap (A-3). Chapter markers and the lower third move post-launch. **V6 stays first, then V6.1-V6.11, then V7-V10**; the releases are renumbered (`technicalspec.md` §42). **The clip path does not change**: a clip of at most 90 seconds with new features off behaves exactly as V5 (`technicalspec.md` §41). The founder's rule: nothing that works is broken without being certain the new thing works, and a feature not worth what it would break goes back to the founder.
- v1: Initial version. Key decisions: one product path (talking-head clip in, vertical short out); four visual-event types, rule-based and precision-first; desktop Chromium only; English only; one paid plan at $15/month; validation experiments run in parallel with the build, not after it; month-6 base-case MRR is about $234, and $1,000 MRR is a month-12+ goal (§13).
- v1.1: Automatic timeline editing is removed from the product. Pause tightening (P0 in v1) and dramatic-pause preservation (P1 in v1) no longer exist. New principle 7 (§8) and scope section §9.2.1: the user provides the take; Offcut enhances the whole take and never cuts it. The only audio transformation is enhancement, which keeps the recording's duration.
- v1.2 (founder decision of 2026-10-21): **Offcut becomes an AI-planned editor.** One cloud LLM call per clip reads the transcript text, word timings and per-word loudness and pitch scores (never audio or video) and returns an **edit plan**: which pauses to trim, which words to emphasize, which lists, numbers, cards, furniture and transitions to show, and the hook title, chapters, post caption and hashtags (§12.8). The rule detector of v1.1 stays as the offline fallback and the first producer of the same plan. Principle 7 is rewritten: Offcut now trims pauses and silences, and every cut is shown and can be restored (§8, §9.2.1). Principle 4 is rewritten: media never leaves the device; the transcript leaves for the plan, disclosed and switchable off (§12.7). Background music with ducking, brand kits, kinetic caption presets, video inside shapes, shape transitions, structure cards and furniture move from P1/P2 into the launch scope as releases V6.1 to V6.11 (§9.5, `buildplan.md` §8A). V4 to V6 are built with the seams these need, so V6.2 starts without rework (`technicalspec.md` §40). Stock-footage B-roll stays out (§21). Launch follows V6.10 and V7-V10; the date is re-set when V6.10 closes (A-2).
- v1.1 (alignment with `technicalspec.md`, no scope change): infrastructure cost is 0 USD/month with no paid fallback, deployed on Vercel (frontend) and Render (backend) (A-3, A-10, §11, §13.6, §15); both OPFS and IndexedDB are required (§9.3); the three-minute promise is stated for the 60-second reference clip (§20.3); analytics are never joined to accounts (§16, §17); account deletion is immediate (§17); the free quota resets on the UTC calendar month and the Creator ceiling never blocks (§11); the loudness target is -14 LUFS (§20.1); one-speaker and English constraints are stated, not detected (§9.4, §20.1); metrics come from SQL queries, not a dashboard (§20.6).

---

## 0. What Changed From Your Original Idea (Read This First)

> **v1.2 note.** Corrections 1 and 2 below were written for v1 and v1.1. v1.2 reverses part of them on the founder's decision: semantic understanding now comes from a **cloud** LLM over the transcript (not an on-device model), checked by a validator that keeps a wrong number off the screen (§12.8), and music, brand kits and more visual primitives are in the launch scope. The corrections stay here as the reasoning they were.

**What is directionally right.** The core instinct is good: for talking-head creators, the speech already contains the edit decisions (what is a number, what is a list, what is emphasized), so a product that reads speech and produces the visual layer is a real wedge against "caption templates." Local-first processing is also right: it makes marginal cost per export close to zero, which allows flat pricing against competitors that meter by credits or minutes. A thin backend that only knows account, plan and usage is the correct shape.

**Corrections**

1. **"It understands what you mean" is not something the MVP can deliver.**
   *Instinct:* a "talking-head compiler" with a story engine that infers intent, importance and structure. *Problem:* real semantic understanding needs an LLM-class model; on-device that is a 1-2 GB-class download with variable performance and hallucination risk, and a wrong graphic is worse than none. *Fix:* the MVP detects four **patterns** with rules and number parsing (spoken numbers, enumerations, from-to comparisons, emphasized keywords), shows an event only above a confidence threshold, and says so honestly in the pitch. Precision beats recall (§8). Local LLM understanding is deferred (f) and gated on E-7 (§19).

2. **Most of the original description is a roadmap, not an MVP.**
   *Instinct:* five engines, 100-200 visual primitives, Style DNA learned from past videos, three versions per recording, remix controls, a semantic story timeline, structure-aware music. *Problem:* none of these is needed to learn whether people want speech-driven shorts, and several are separate products. *Fix:* one path, 4 event types, 3 caption styles, no music. Cuts and codes: Style DNA (d)(a); three versions (a); remix controls (a)(e); semantic timeline editor (d)(f); structure-aware music (a); bundled music library (c) licensing; user-supplied music is P1 (e) for the first test; 100+ primitives (a): four types answer the hypothesis, E-7 tells us which to add.

3. **The numbers in the original pitch are not realistic, and you will not like this.**
   *Instinct:* "$1k MRR by month 3" and a 130k-230k LOC "serious v1." *Problem:* with any defensible funnel for a free-tier-plus-paid tool, month-6 MRR is about **$234 in the base case** and $1,000 MRR requires roughly **51,400 visitors in six months** (§13); the LOC table in the source is not a decision input and I am not using it (§9.9). *Fix:* month-6 base case and month-12+ goal are stated plainly; the build is sized by capabilities, not by lines. If $1k by month 3 is non-negotiable, this product at this price does not deliver it.

4. **"Everything on device" is a strength with three costs the idea ignored.**
   *Instinct:* local STT, local render, no media servers. *Problem:* (i) reach: desktop Chromium only at launch, while many creators film and edit on phones; (ii) support: your CPU/GPU/browser matrix is now the user's, and odd devices become your support tickets; (iii) privacy is probably a tiebreaker, not the headline reason people pay. *Fix:* explicit platform scope (§9.3), capability detection with honest unsupported pages, a reach experiment (E-8) and a privacy-messaging experiment (E-6). FFmpeg compiled to WASM as the export engine is dropped from the MVP: WebCodecs plus a Rust MP4 muxer is smaller and avoids FFmpeg's licensing review burden (c)(f); the tech spec decides the detail.

5. **Pricing from the source ($19 / $49 / $99) over-specifies tiers you cannot yet justify.**
   *Instinct:* three paid plans from day one. *Problem:* Pro and Studio features (three versions, brand systems, teams) are cut above; selling plans with nothing in them is a trust problem, and reported competitor prices (§7) put a single $15 plan in a defensible spot. *Fix:* Free + one paid plan (Creator, $15/month) at MVP. Pro at $29 is P1 and is introduced only when Pro features exist (§21).

6. **The validation order is backwards.**
   *Instinct:* build the engine for a week, make the demo amazing, then find customers. *Problem:* the scarce resource is validation, not typing (see Core Philosophy item 7). A 20k-line week can still produce a product nobody pays for. *Fix:* in Month 0, the demand smoke test (E-1), a feasibility spike (E-3, E-4) and a paid presale (E-5) run alongside the build, and a go/no-go gate at M0 (end of week 3) can stop the build before weeks 4-10 are spent.

7. **The "Rust/WASM/Vello" story is implementation, not positioning.**
   *Instinct:* lead with the engine. *Fix:* the pitch is the outcome ("turn what you say into a finished short"); Rust/WASM/Vello are why it is private and fast, which is a supporting claim (H4).

### 0.1 Inputs and explicit assumptions

No clarifying questions were needed to write the spec; these assumptions are explicit and each can be corrected in v2.

| ID | Assumption (all ASSUMPTION unless noted) |
|---|---|
| A-1 | Founder is a solo Rust backend engineer (Axum, Tokio, SQLite/Postgres, Redis), comfortable with React/TypeScript, using coding LLMs heavily; throughput of roughly 20k lines of working code per week is plausible for well-specified modules (founder's own estimate), but integration and debugging time are separate (§9.9) |
| A-2 | 50 working hours per week; build starts Monday 12 Oct 2026; Month 0 = weeks 1-10 (ends Sun 20 Dec 2026); public launch Mon 21 Dec 2026 (Month 1 start). **v1.2:** V6.1-V6.11 sit between V6 and V7 (`buildplan.md` §8A). V1-V3 finished about two weeks ahead of plan (V3 closed 2026-10-21 against 4 Nov), and that slack goes to V6.x first. The 21 Dec date holds only if V6.x fits in it; otherwise M2 and the launch move by the overrun, and the founder re-sets the date when V6.11 closes (v1.3: about February 2027, ESTIMATE). V7-V10 keep their five weeks |
| A-3 | Cash budget is $0/month for infrastructure and tools, **except the AI provider (v1.3): paid is allowed, under a monthly spending cap the founder sets**; zero paid acquisition. At the cap, AI requests stop until next month and the local rules take over, so no clip or video is blocked (§11) |
| A-4 | Founder is India-based and sells globally in USD through a merchant of record (Paddle-, Lemon Squeezy- or Dodo Payments-class) rather than a direct card processor, to avoid cross-border tax handling (general-market knowledge, re-verify before launch) |
| A-5 | Founder's own distribution is public build-in-public posting plus direct outreach; no existing paying audience |
| A-6 | MVP language is English (US, UK, Indian accents); no Hindi/Hinglish |
| A-7 | Reference hardware R1: 2021-class Windows laptop, 8 GB RAM, integrated GPU, Chrome stable, plugged in. R2: Apple M1, 8 GB, Chrome stable |
| A-8 | A typical user has 25 Mbps or better download speed for the one-time model download |
| A-9 | Merchant-of-record fee is about 5% + $0.50 per transaction (general-market knowledge, re-verify before launch) |
| A-10 | Fixed infrastructure cost, apart from the capped AI provider (A-3), is $0/month with no paid fallback: frontend on Vercel's free plan, backend on a Render free web service, plus free tiers for Postgres, model/asset storage, transactional email and uptime monitoring (each verified by a tech-spec experiment). No domain is purchased. When a free tier stops fitting, the component moves to another free tier, not a paid one |
| A-11 | Product name "Offcut" is a working name |
| A-12 | Customers: YouTubers who publish 10-45 minute talking-head or explainer videos (v1.4; v1.3: 10-20) and edit on a desktop, and creators who film short talking-head clips on a laptop or transfer phone clips to one (v1.3: equal on the landing page; E-14 reads which pays) |

---

## 1. Executive Summary

**What it is.** Offcut is a browser-based tool that turns a raw talking-head clip into a post-ready vertical short. Speech recognition, audio cleanup, cutting, music mixing, rendering and encoding run on the user's machine. The server handles accounts, billing, usage counts and, from V6.2, the edit-plan request: it forwards the transcript text and timings to an LLM and returns the plan, storing none of it (§12.8).

**For whom.** Technical creators and indie founders (developers, dev-rel, SaaS builders) who post short talking-head clips full of numbers, benchmarks, lists and before/after claims, and who currently spend 30-60 minutes per clip in a general editor.

**The MVP in one pipeline (v1.2).**
`raw clip (≤ 90 s) → local transcription → edit plan (cloud LLM over transcript text and timings; local rules when offline or switched off) → validation → enhanced voice → cuts + music with ducking (program audio) → kinetic captions, structure visuals, furniture, transitions, brand kit → review (edit words, toggle items, restore cuts) → 1080×1920 MP4`

**Architecture / business-model thesis.** Local processing means near-zero cost of goods per export, so one flat monthly price with unlimited normal use is possible against competitors that meter by credits or minutes (reported, §7). The thin server is the only recurring cost.

**The long path (v1.3).**
`long video (≤ 45 min, v1.4, read in place) → cloud transcription (resumable) → local vision signals → Director (AI over sentences; local rules if unavailable) → 5-10 shorts of whole sentences, stitched if that makes the point → for each: the pipeline above, plus face-tracked framing → list of shorts → keep, edit by clicking or chat → export the ones you keep`

**The wedge (three bullets).**
- What you said decides the edit: lists, numbers, comparisons, chapters and topic shifts become visuals, and pauses that carry nothing are trimmed, all from one plan that never puts a number on screen that you did not say.
- Raw footage and voice are not uploaded, which matters for unreleased product demos and client material (privacy is a hypothesis, H4, not a given); only transcript text and timings go out, for the plan.
- Flat price, bigger quotas than credit-metered competitors, no per-minute metering: rendering costs us nothing because it runs on your machine.

**Core business hypotheses the MVP exists to test**

| ID | Hypothesis |
|---|---|
| H1 (value) | For clips ≤ 90 s, auto-generated captions plus visual events get a creator to a postable short with ≤ 2 minutes of review; ≥ 50% of activated users would post the result with minor edits (TARGET) |
| H2 (willingness to pay) | Technical creators will pay $15/month for H1, despite CapCut being free and competitors being reported at roughly $10-$39/month |
| H3 (local viability) | A typical modern desktop Chromium browser runs the full pipeline within the time budget in §20.2 without server compute |
| H5 (long-path value, v1.3) | YouTubers post at least half of the shorts the Director proposes, with ≤ 2 minutes of change each, and will pay $15/month for it (TARGET; E-12, E-13) |
| H4 (privacy as pull) | "Your video is never uploaded" raises signup or paid conversion, or at least is not a drag; it is a purchase driver rather than only a tiebreaker |

**Honest target numbers (all ASSUMPTION-based; formulas in §13).**

| Target | Value | Month | Requires |
|---|---|---|---|
| Base-case MRR | about $234 (16.4 active paid) | Month 6 | about 12,000 visitors, 4% visitor-to-signup, 4% signup-to-paid, 8% monthly churn |
| Stretch MRR | about $1,162 (79.9 active paid) | Month 6 | about 25,000 visitors, 6% / 6%, 6% churn |
| $1,000 MRR at base rates | about 70 active paid | Month 12+ goal | about 51,400 visitors in six months at base rates (§13.5) |

---

## 2. Problem

| Current alternative | What it gets right | Weakness (re-verify; *general-market knowledge*) |
|---|---|---|
| **CapCut** (free core; Pro reportedly about $8/month) | Free, fast, huge template and effect library, auto-captions | Manual timeline work: cutting, caption placement, per-word styling, animating numbers. Time cost, not money, is the pain; ByteDance ownership concerns some users |
| **Submagic-class caption tools** (Starter reportedly $19/month, Pro $39/month) | Good animated captions, silence removal, B-roll, per-export credits | Cloud upload; paid tiers; generic styling not tied to what is said (a number is styled like any word); credit-based exports |
| **Opus Clip** (reportedly Starter $15/month for 150 source minutes; Pro $29/month for 300) | Strong at clipping long videos into shorts, on servers, so the laptop can close | Credits tied to source minutes, which punishes mining a back catalogue; reviews report 20-40% of clips discarded, often for mid-sentence cuts (*third-party, re-verify*); standalone clips rather than one point built from several moments (*not confirmed, E-12 tests it by hand*); little value for a clip that is already short |
| **Captions** (Pro reportedly $9.99/month) | All-in-one creation studio, strong for talking-head posts, avatars | Cloud processing; broad studio, not specialized in visualizing spoken claims |

**Underserved pain.** Technical creators say numbers, benchmarks and lists out loud ("latency went from 800 ms to 120 ms", "three reasons") because that is how they think, then spend most of their editing time making those moments visible. Existing tools treat every word as equal. The pain is not captions; it is the manual step between "I said a number" and "the viewer sees the number land." **A third pain (v1.3).** YouTubers already have 10-20 minutes of good talking on file, and filming separately for Reels and Shorts is a second job. Cutting shorts by hand from the long video takes hours, and clip tools give a pile of fragments that still need fixing. A second, smaller pain is that people who handle unreleased product footage or client material are uncomfortable uploading raw video to a third-party SaaS.

---

## 3. Target Customer

**Two equal launch audiences (founder, 2026-11-02).** The landing page speaks to both; each has its own page from v1.4 ("Make shorts from a long video", "Edit a clip"). E-14 reads which one pays, and marketing follows that evidence.

**Long-path ICP (v1.3; v1.4).** Solo YouTubers who publish 10-45 minute talking-head or explainer videos (talks and solo podcasts often run past 30), one speaker or two on screen (v1.5: two-person podcasts, the frame follows whoever speaks), in English, edit on a desktop, and want Shorts, Reels and TikToks without filming twice. About 1,000 to 500,000 subscribers (ASSUMPTION). Budget: $15-$30/month (ASSUMPTION, E-5). They have master files on disk, often 4K.

**Primary ICP (MVP serves this one; widened in V4, the founder's decision of 2026-10-22).** Short-form creators who film talking-head clips, 2-5 per week, technical or not, for X, LinkedIn, YouTube Shorts, Reels or TikTok. Technical creators and indie founders, who talk about software, metrics, benchmarks, launches or lessons learned, stay the first distribution channel: their speech is full of the numbers and lists the four MVP visuals show. As first written: technical creators and indie founders making 2-5 short talking-head videos per week for X, LinkedIn, YouTube Shorts, Reels or TikTok about software, metrics, benchmarks, launches or lessons learned. Solo or two-person operations. Budget ceiling for a video tool: $15-$30/month (ASSUMPTION; tested by E-5). Comfortable with desktop Chrome, file uploads and account sign-up. Currently use CapCut, a caption tool, or DaVinci/Premiere for occasional polish.

**Secondary ICP (not served by the MVP).** Coaches, consultants and LinkedIn thought-leadership creators who record talking-head advice. They care about the same time savings but their speech has fewer numbers and lists, so the four MVP event types help them less. Plausible price: $15-$29/month. Target them when P1 event types (quote card, warning, question, checklist) ship and E-7 shows event keep rate ≥ 60% (§21, item 4).

**Not targeted at launch:** agencies and teams (d); podcasts with three or more people on screen at once (f) (v1.5: two people are in, §12.10; v1.3 said one speaker at launch); movie and game reviewers whose videos are mostly other people's footage (copyright, and little speech-on-camera to frame) (c)(f); non-English creators (f)(e).

---

## 4. Persona

*Illustrative composite for design decisions, not a real customer.*

**Daniel Reyes, 32,** founder of a one-person developer-tools SaaS in Austin. Ex-backend engineer. Posts 3 clips per week on X and LinkedIn: build updates, benchmark results, "what I learned" lessons, 30-60 seconds each, filmed on a laptop webcam or phone and AirDropped to his laptop.

- **Workflow:** records 1-3 takes; imports into CapCut; trims silence by hand; auto-captions; fixes caption errors; tries to make a number "pop"; resizes to 9:16; exports; sometimes re-exports after noticing a bad line break. About 45 minutes per clip.
- **Current tools:** CapCut, occasionally Descript for audio; free tiers where possible.
- **What annoys him:** time spent on mechanical steps; captions that treat "$2k to $20k" like any other words; fear of uploading unreleased product footage to a random web app.
- **Constraints:** posting is not his job; budget is his own card; he will try a tool for 15 minutes and abandon it if the first output is not clearly good.

**Maya Okafor, 27** (added in V4: the audience is creators in general), a fitness coach in Manchester who posts four Reels and Shorts a week from her phone: tips, routines, "three mistakes beginners make". Not technical. She uses CapCut's auto-captions and fixes them by hand, and wants captions that look like the creators she follows without learning an editor. Every word on Offcut's screen must make sense to her without help: "Visuals", "Captions", "Style", "Framing", "Export"; never "detector", "scene", "plan" or "LUFS" outside the settings page's technical note.

---

**Arjun Menon, 29** (added in v1.3), an explainer YouTuber in Bengaluru with 80,000 subscribers: one 15-minute video a week, shot in 4K with a mirrorless camera and a good mic, edited in DaVinci. Wants Shorts and Reels every week and does not want to film them separately. Tried a clip tool: half the clips ended mid-thought and needed fixing, and credits ran out when trying older videos. Happy to leave a video running while cooking dinner, not happy to learn another timeline. Would pay $15 for "five shorts I'd actually post" from each video.

---

## 5. Jobs To Be Done

1. When I have recorded a raw talking-head clip, let me get a postable vertical short in minutes, so that editing does not eat the time I need to build the product.
2. When I say a number, a list or a before/after claim, let the video visualize it automatically, so that viewers catch the point without me animating anything.
3. When my footage contains something unreleased or confidential, let me edit it without uploading, so that I do not have to trust another vendor with it.
4. When the automatic result has a mistake, let me fix a word or turn off a visual in seconds, so that I never have to learn a timeline to correct it.
5. When I post several clips per week, let my shorts look consistent and clean without per-clip styling decisions, so that my feed looks deliberate.
6. When I am deciding whether to pay, let me see the real output on my own clip first, so that I never buy on a promise.
7. (v1.3) When I publish a long video, let me get several finished shorts from it without filming again or fixing fragments, so that one video feeds a week of posts.
8. (v1.3) When the point I want is spread across my video, let the short join those moments, so that it makes sense on its own.
9. (v1.3) When something is not right, let me say what I want in words, so that I never need to learn editing.
10. (v1.3) When I have years of old videos, let me mine them all without counting minutes, so that old work earns new views.

---

## 6. Value Proposition

**v1.3.** For **YouTubers with long talking-head videos and creators who film short ones**, Offcut is a **browser-based short-form video maker** that **turns a long video into several finished shorts, or a raw clip into one, without uploading your video and without a timeline**, unlike **clip tools (uploads, per-minute credits, fragments to fix) and CapCut (manual work).**

*v1.2 wording:* for **creators who post short talking-head videos** (V4; first published for technical creators whose videos are full of numbers and lists, still the first channel), Offcut is a **browser-based short-form video maker** that **turns a raw clip into a captioned, cleaned-up vertical short with automatic visuals for what you said, without uploading your video**, unlike **CapCut (manual timeline work) and cloud caption tools (upload, credits, generic styling)**.

---

## 7. Differentiation & Alternatives

| Alternative | What it solves | Weakness this product exploits |
|---|---|---|
| CapCut | Free full editor, templates, auto-captions | Manual effort per clip; no speech-driven visuals for numbers/lists |
| Submagic-class tools | Animated captions, silence removal, B-roll | Cloud upload; credit metering; styling not driven by what is said |
| Opus Clip, Submagic Magic Clips, Vizard | Clips from long-form video, on servers | Minute or upload metering; mid-sentence fragments; little that follows what was said. **Their strengths: speed, and the laptop can close.** Offcut will not match those |
| Captions | Talking-head studio with avatars and effects | Cloud processing; broad studio feel; not specialized for claims and numbers |
| Doing nothing (post unedited) | Zero effort | Lower retention on short-form platforms (ASSUMPTION; not tested here) |

*Competitor features and prices come from third-party 2026 articles and aggregators (some of which conflict, for example on free-plan availability) and are **general-market knowledge, re-verify before launch** on each vendor's own pricing page. Competitors may add speech-driven visuals or on-device processing at any time (§18).*

**What is not a moat:** Rust, WASM, Vello, Whisper-class models, "runs locally" and "long video to clips" all exist elsewhere or can be copied. Clipping long videos is OpusClip's home ground; adding it makes Offcut a real alternative, not a defensible one by itself.

**Where Offcut can actually win (v1.3).**

| Advantage | Honest strength | Depends on |
|---|---|---|
| A short built from several moments, in a sensible order | Possibly unique; hand-test it (E-12) | The Director's judgement (E-13) |
| Clips that always start and end on whole sentences | Strong, and structural | Sentence segmentation |
| Instant edits by chat or click, no re-render | Strong: 22-42 ms edit-to-preview today | The preview holding that on heavier scenes |
| An unmetered back catalogue | Strong: credits make this expensive elsewhere | Overnight local runs surviving (TE-20) |
| Visuals that follow what was said, and numbers that are never invented | Medium: good, and visible | Catalog quality |
| Sharp face-tracked crops from the 4K master | Medium | 4K decode on real machines |
| Privacy | Tiebreaker (H4, E-6) | |
| Flat price for a weekly uploader | Weak: one video a week fits inside OpusClip's Starter | |

**The moat to build is the creator's own history.** Creator memory (§12.14) makes each plan fit better the longer someone stays, and later, performance data can teach it what worked (§21 item 18). This is why it is launch scope.

---

## 8. Product Principles

1. **Time to AHA beats feature count.** The user must see their own clip animated within minutes of dropping it. If a feature adds a step before the first preview, it is cut or made optional.
2. **Precision beats recall for visual events.** Show no graphic rather than a wrong graphic. A missed list is a shrug; a wrong number on screen is a lost user. Rule-detector thresholds are tuned to ≥ 90% precision (TARGET) even if that means fewer events. For the AI edit plan the same principle is structural: the model never writes a number; it points at the words that said it, and the value shown is parsed from the transcript (§12.8). A plan item that fails validation is dropped, not repaired.
3. **Defaults beat configuration.** One default style, one voice chain, no sliders. The plan decides; the user reviews with on/off toggles: per plan item, per cut ("restore this pause"), per part of a short, music on/off, and a brand kit chosen once, or says what they want in chat (v1.3). No timeline, no keyframes. **(v1.5) The Enhance panel:** one switch per enhancement (voice cleanup, motion, colour and light, sound effects, music), each measured first so it does only what the video needs; switches, never sliders (§12.16). (v1.4: placing B-roll by hand on a word, and choosing which part of a clip plays by dragging a window along its frames, is the one direct control besides the crop offset, §12.12.)
4. **Media never leaves the device; what does leave is stated in the UI, and a test proves it.** Video never leaves. Sound leaves only to be transcribed: a clip's while Cloud is chosen in the drop zone (cloud transcription release, §12.6), and a long video's always (v1.4, §12.9), compressed, passed to the transcription provider and kept by neither. From V6.2 the transcript text, word timings and per-word loudness/pitch scores are sent to our server and the AI provider named on the settings page, only to make the edit plan, and are stored by neither us nor (per TE-15) the provider beyond the request. The "AI edit plan" switch in settings turns it off and falls back to local rules, after which nothing but account and analytics traffic leaves (§12.7). **v1.3:** the long path sends sentence text and timings to choose shorts, chat sends the typed message with that short's words, and asset labels the user typed go with plan requests: all only with the switch on, none stored. Local vision sends only categories such as "talking head, face left", never a frame. The network test (§17) blocks any other regression.
5. **The free tier must be genuinely usable.** Full-length processing and preview, 3 exports per month at 720p with a small watermark. Paywalls are shown before the action, never after the work (§10).
6. **Complexity must buy a hypothesis, and may not break what works (v1.3).** New features are added beside the working product. A feature not worth what it would break goes back to the founder (`technicalspec.md` §41). Every subsystem has a row in §9.7 and §9.8. "It is a lot of code" is not a reason to cut; "it does not test H1-H4" is.
7. **The take is the user's; the cut is a suggestion the user can see.** (Rewritten in v1.2.) Real speech is not its transcript: a pause can be dramatic or dead air, a silence can be a beat, "first" can start a list or not. The edit plan judges this from the whole transcript with its timings and prosody, and proposes cuts: trimmed lead-in and tail silence, shortened pauses, and (off by default) filler words and false starts. Every cut is listed in review and can be restored with one toggle. A cut never starts or ends inside a word, a pause is shortened rather than removed (a breath of at least 150 ms stays, TARGET), cuts are snapped to the quietest point and cross-faded, and jump cuts are hidden with an alternating punch-in. With the plan off and no cuts, the export is exactly as long as the clip that was dropped in, as in v1.1 (§9.2.1). **v1.3:** a short from a long video is whole sentences only, its parts are listed with their words, any part can be removed and restored, and a moved hook must mean the same in its new place (§12.9). The model never invents a claim by joining sentences that change meaning; E-13 reads this.

---

## 9. MVP Scope (P0) - Ruthless Product Scope

Ruthless here means eliminating unnecessary product capabilities. It does not mean minimizing implementation complexity.

### 9.1 Product scope (what the user can do)

```text
Open site → drop raw clip (≤ 90 s, English) → [first run: one-time model download]
 → local transcription → edit plan (AI over transcript text + timings; local rules if off/offline)
 → voice enhancement → program audio (cuts, music, ducking)
 → preview (kinetic captions, structure visuals, furniture, transitions, brand kit; v1.5: motion, colour and light, sound effects)
 → review: edit words / toggle items / restore cuts / music on-off / pick style / brand kit / Enhance switches (v1.5) / copy post text
 → [sign in if needed] → export 1080×1920 MP4 (free: 720×1280 + watermark)
```

**The long path (v1.3, from V6.4; v1.4; full flow in §12.9):**

```text
Open site → "Make shorts from a long video" → drop it (≤ 45 min, English, one or two people on screen) → sign in (limits shown first)
 → read in place, cloud transcription, resumable → local vision signals → Director picks 5-10 shorts
 → only the chosen parts are kept → list of shorts: poster, title, length, why
 → open one: the same review as above (plan applied, B-roll by hand), plus chat → export the ones you keep (each counts as one export)
```

Both paths share everything after the short exists. There is no timeline, no project library beyond "recent clips" in browser storage (the 5 most recent, shown by date and duration; file names are not stored), and no collaboration.

### 9.2 Explicitly not in the MVP

| Item | Reason code(s) and why |
|---|---|
| Style DNA (learn style from past videos) | (d) a second product; (a) not needed to test H1 |
| Three versions per recording / remix controls | (a) H1 does not require variants; (e) not needed for the primary workflow |
| Semantic "story" timeline editor and general timeline | (d) it is an editor, a different product; (f) high cognitive and maintenance cost |
| Bundled music library | Founder decision 2026-11-09 (v1.5): the person's own track only at launch; a bought library is §21 item 23. User-supplied music with ducking is **in** (V6.7). Sound effects are **in**, synthesized in code, so they need no licence (§12.16) |
| Local LLM for deeper semantic understanding | (f) download size, device variance; the v1.2 plan uses a cloud LLM over the transcript instead (§12.8). A local model stays P2 as the offline producer of the same plan |
| Cloud transcription (audio upload) | **Built in the cloud transcription release** for signed-in people, chosen in the drop zone's card (§12.6); the video is never sent. **v1.4: always on the long path** (§12.9). The cloud **LLM over transcript text** is in from V6.2 (§12.8) |
| Languages other than English (including Hindi/Hinglish) | (f) separate accuracy testing, font shaping and caption rules per language; (e) not needed by the primary ICP |
| Phones and tablets (browser) | (f) memory limits, WebCodecs/WebGPU variance; *reach is a known risk*, measured by E-8 |
| Safari, Firefox | (f) test matrix and encoder/GPU differences; P1 gated on E-8 |
| Linux desktop | (f) reported lack of AAC encoding in browsers on desktop Linux (*general-market knowledge, re-verify*) |
| 4K output | (f) memory and time budgets; (e) vertical shorts do not require it. **4K input is in (v1.3):** a face-tracked vertical crop from it is sharper |
| 100+ visual primitives | (a) the v1.2 catalog (§12.8) is the launch set; additions are catalog entries, measured by E-11 |
| AI-generated B-roll, AI avatars | (d) second product. **The user's own B-roll is in (V6.8), and v1.4 adds Pixabay stock footage (V6.8, §12.12):** its licence allows commercial use, its CDN works under COEP, and searches go through our server. *(v1.3 kept stock out for those three reasons; the founder reversed that on 2026-11-08.)* |
| Sending video frames to the AI | Founder decision 2026-11-02: it changes the privacy promise; local signals only. A possible later opt-in if E-13 shows clip choice needs it |
| YouTube-link import | (c) a server that downloads video breaks "not uploaded" and platform terms; creators have the file |
| Virality score | (c) unverifiable and doubted by reviewers; each short shows its reason instead |
| Three or more people on screen at once; finding speakers by voice | (f) a second tracking and diarization product. **v1.5 (founder, 2026-11-09): two people on screen are in (V6.5):** the frame follows whoever is speaking, found by matching faces to speech on the device, with a split screen when unsure (§12.10). *(v1.3: one speaker at launch.)* |
| HDR video (the iPhone default) | Founder decision 2026-11-09 (v1.5): rejected with advice to record or export SDR. The browser's own HDR mapping crushed saturated colours in TE-21, and our own tone mapping is not worth its cost before launch (§21 item 22) |
| Chapter markers, lower third | Founder decision 2026-11-02: moved post-launch; made for longer videos, rarely useful in a short (§21 item 15) |
| Multi-brand, teams, shared styles | (d)(e) not needed by solo creators. One brand kit per account is **in** (V6.10) |
| Social scheduling, analytics integrations | (d) |
| Public API, native desktop app | (d); native is P2 after the web product validates |
| Pro ($29) and Studio plans | (a) nothing exclusive to sell yet; (d) team features |
| Rule-only automatic timeline editing | (f) whether a pause is intentional, dramatic or dead air is a semantic judgment that rules cannot make; v1.2 makes it with the edit plan (§12.8). Without a plan (switched off, offline, quota spent) Offcut cuts nothing |
| Removing whole sentences or retakes **inside a clip by automatic plan** | (f) a wrong cut of meaning is worse than none; the edit plan may only trim silence, shorten pauses and (off by default) drop filler words and false starts (§9.2.1). A long video's Director selects whole sentences as the point of the feature, and every part is shown (§12.9). A user may ask chat to cut a sentence (§12.11) |

#### 9.2.1 What Offcut does to your recording (v1.2)

From V6.3 the edit plan proposes, and the review shows, these cuts:

| Cut | Default | Rule |
|---|---|---|
| Silence before the first word and after the last word | On | Kept to a short lead-in and tail (TARGET 250 ms and 400 ms) |
| Pauses between words | On | Shortened, never removed: at least 150 ms of the pause stays (TARGET); a pause the plan marks as intentional (a beat before a punchline, a dramatic stop) is kept whole |
| Filler words ("um", "uh") and false starts | **Off** | Proposed in the plan; applied only if the user switches them on per clip |

Inside a clip's recording, Offcut does **not**: remove whole sentences or retakes on its own, reorder anything, change speaking speed, or cut inside a word. (v1.3: choosing and ordering whole sentences from a long video happens before this, and is listed in review, §12.9.) Every cut is shown in review with its length and can be restored; the "What we changed" summary states the time removed. Cuts are snapped to the quietest 10 ms around the boundary and cross-faded so no click is heard; the video gets an alternating punch-in zoom at each cut so jump cuts read as deliberate.

**Without a plan, nothing is cut.** With the AI edit plan off, offline, or out of quota, the local rules make captions and visuals only: the export is exactly as long as the dropped clip and pauses stay where they were recorded, as in v1.1.

**Audio enhancement is still separate from editing.** The voice chain (denoise, high-pass, compression, loudness, limiting) keeps the recording's duration; cuts and music are applied afterwards, in one step that builds the program audio (`technicalspec.md` §40). **(v1.5)** The chain measures the recording first, so a clean recording gets a light touch, and "Voice cleanup" can be switched off in review: the sound is then only levelled to the platform loudness (§12.16).

### 9.3 Supported platform

- **Supported:** desktop Chrome and Edge (current stable) on Windows and macOS. ASSUMPTION: these cover most of the primary ICP's laptops; E-8 measures.
- **Detection:** feature detection, not user-agent string parsing. On page load the capability check requires: `VideoEncoder`/`VideoDecoder` with `isConfigSupported` true for H.264 decode and for H.264 encode at 1080×1920, and AAC audio decode and encode; WebGPU adapter available; WebAssembly SIMD and threads (cross-origin isolation); both OPFS and IndexedDB storage (large files and metadata); at least 4 GB device memory where reported; a desktop form factor (the structured `navigator.userAgentData.mobile` hint, since phones and tablets are out of scope, §9.2). Tests run in under 3 seconds and the result is logged as an allowlisted event (§17). (V6, founder's decision of 2026-11-03: a storage check that has not answered is asked once more, so on a slow disk the check can take up to 4 seconds rather than wrongly telling a capable browser it cannot keep files.)
- **Unsupported users see:** a plain page stating the exact missing capability ("Your browser cannot encode H.264 video here"), the supported list ("Chrome or Edge on Windows or macOS"), a link to the demo video so they still see the product, and an email field "Tell me when Safari/Firefox/mobile is supported." Counted as `unsupported_reason`.
- **Deferred and why:** Safari, Firefox, Linux, mobile (see §9.2). WebCodecs reportedly reached full cross-browser support only in 2026 (Safari 26.x) and Firefox for Android lacks it, with patchy audio-encode support (*general-market knowledge, re-verify before launch*); that makes a Chromium-first launch the lowest-risk way to test H1-H3.

### 9.4 Hard input/output constraints

| Dimension | Clip path ("Edit a clip") | Long path (v1.3, V6.1; v1.4: "Make shorts from a long video") | Explicitly deferred |
|---|---|---|---|
| Container / codec | MP4 or MOV, H.264 **or HEVC where the browser decodes it (v1.3)**, AAC audio | Same | ProRes, AV1, WebM, MKV, MP3-only audio |
| Duration | ≤ 90 s | Over 90 s, up to **45 minutes** (v1.4; v1.3: 20). A file of 90 s or less dropped here opens as a clip | Over 45 minutes (f); E-14 reads demand |
| File size | Up to 16 GB with 4K input on (v1.3; 500 MB with it off) | **No limit** (v1.4): the file is read where it is, never copied whole; a small disk check covers the extracted sound and the chosen parts | |
| Resolution | Up to 3840 px on the long side (v1.3; 1920 with the flag off) | Same | Above 4K (f) |
| HDR | Rejected with advice (founder, 2026-11-09, v1.5) | Same | HDR input (§21 item 22) |
| Frame rate | Up to 60 fps input; 30 fps output | Same | 60 fps output (f) |
| Orientation | Portrait passes through; landscape: face-tracked 9:16 crop (V6.5), with a draggable offset | Same | |
| Speakers | One, or two on screen at once (v1.5, V6.5): the frame follows whoever speaks | Same | Three or more on screen; speakers by voice (f) |
| Language | English | English | All other languages (f)(e) |
| Audio tracks | One | The first track, if several | Multi-track choice |
| Output | MP4 H.264/AAC, 1080×1920 at 30 fps (Free: 720×1280 with watermark) | Same, per short: 15-90 s | 4K, 1:1/16:9 output, other containers |

**Violations show a specific, actionable rejection, never a generic error.** Each message states the limit and the advice, not the clip's measured size or duration (decided 2026-10-15; `technicalspec.md` §15.3, v3implementation D-80). Examples (v1.3 keeps each while its flag is off; v1.4: the clip page keeps them, and the long page states 45 minutes and has no size message; with the long path on, the clip page's "too long" message adds a button, "Make shorts from it"): "This clip is too long. Offcut handles clips up to 90 seconds right now. Trim it in your phone's gallery or in your editor, then drop it again." / "This video is larger than Offcut handles. Offcut handles video up to 1920 pixels on its long side. Record or export at a lower resolution, then drop it again." / "This file uses HEVC, which this browser cannot decode here. Export as H.264 or use 'Most Compatible' in your camera settings." Every rejection is counted as `reject_reason` (analytics allowlist, §17) with no file name, content or transcript.

**Two constraints are stated, not detected.** At most two people on screen (v1.5; v1.3: one speaker) and English are shown at the drop zone (J2) but are not checked at import: a clip of more people is processed as it is, and non-English speech usually ends in "no English speech found" or poor captions. A clip with fewer than 3 transcribed words is rejected as having no speech. Fragmented MP4 files are rejected as an unsupported container at MVP.

### 9.5 Feature table

| Feature | Priority | Why / journey link (§10) / promotion rule |
|---|---|---|
| Capability detection + unsupported page | P0 | J3. Without it, unsupported users fail silently and E-8 cannot be measured |
| Input probe + actionable rejections | P0 | J5. Odd real-world inputs are the main stabilization risk |
| One-time model download with progress and explanation | P0 | J4. First-run time is part of time-to-value |
| Local transcription with word timestamps | P0 | J6. Captions and events depend on it (H1, H3) |
| Pattern detector: spoken numbers, enumerations, from-to, emphasized keywords | P0 | J6-J7. The wedge (H1) |
| Voice cleanup (denoise, high-pass, compress, limit) + loudness normalization; the recording's duration is unchanged | P0 | J7, J10. Audibly better output is half the perceived quality |
| Animated captions, 3 styles (Clean, Bold, Tech) | P0 | J7-J8. The baseline feature competitors already have |
| 4 visual events: NumberReveal, ListReveal, FromTo, KeywordPop | P0 | J7. The differentiator (H1) |
| 9:16 framing (portrait pass-through; landscape static crop + offset) | P0 | J7-J8 |
| Preview player with synced audio | P0 | J7. The AHA moment lives here |
| Review UI: edit caption words, toggle events, pick style | P0 | J8. Without it any ASR error is a dead end (principle 3 allows toggles only) |
| Export 1080×1920 MP4 (Free 720p + watermark) | P0 | J10 |
| "What we changed" summary | P0 | J11. Builds trust, supports E-7 |
| Account (magic-link sign-in), entitlement cache, usage receipt | P0 | J9. Needed for free-tier limits and billing |
| Checkout via merchant of record, Creator plan | P0 | J9, J12. Needed for H2 |
| Settings page "What leaves your device" | P0 | J1, J4. Required for H4 and trust (§12.7) |
| Allowlisted analytics + network-privacy test | P0 | All steps. Required to measure activation and to prove the privacy claim (§17) |
| **Long source** (its own page; 45 minutes, read in place, 4K and HEVC input, cloud transcription in resumable windows; v1.4) | P0 (V6.1) | §9.4, §12.9. Foundation of the long path; first because it holds the biggest feasibility risk |
| **AI edit plan** (cloud LLM over transcript text, timings and prosody scores; validated; applied automatically; local rules as fallback), with emphasis picks (colour, icon pop, **auto zoom**: a snap or a slow push on the key words, v1.4), transcript-aware emoji, and text extras (hook title, post caption, hashtags); **(v1.5) the hook title drawn as a lockup over the first 2 seconds, caption glow, punch words at twice the size, emoji under the keyword, gentle motion on every talking-head shot by rules, and the Enhance panel (voice cleanup and motion switches)** | P0 (V6.2) | The backbone every later release reads (§12.8). Founder decision 2026-10-21; emoji kept 2026-11-02; auto zoom 2026-11-08; the benchmark's motion and captions 2026-11-09 (§12.16) |
| Intelligent pause and silence trimming (cuts proposed by the plan, restorable in review), **and stitched parts** | P0 (V6.3) | §9.2.1, §12.9. Raises the quality of the final short more than any visual (founder) |
| **Director: a long video becomes 5-10 shorts** of whole sentences, optionally stitched from different parts; list page; batch export | P0 (V6.4) | §12.9. The headline feature of the long path |
| **Local vision and face-tracked framing; (v1.5) following whoever speaks in a two-person shot, with a split screen when unsure** | P0 (V6.5) | §12.10. A long landscape video is unusable as a short without it, and a crop on the wrong person of a podcast looks broken (founder, 2026-11-09) |
| **(v1.5) Picture:** automatic exposure, white balance and contrast per shot, matching cameras, a soft light on the face, a light vignette; a blur used by layouts and cards; a "Colour & light" switch | P0 (V6.6a) | §12.16. Founder, 2026-11-09: what Tooscut-class engines do that server tools like Submagic do not; each correction is measured and bounded, so a good picture is left alone |
| **Finished look (Graphics):** list builder, number moments, cards (quote, X-versus-Y, definition, step tracker, timeline, checklist), kinetic caption presets, (v1.5) four display fonts, six presets with our own names and 1-word or 1-3-word captions, video inside shapes, shape transitions, progress bar, end card with a call to action | P0 (V6.6b) | Founder kept all of these on 2026-11-02, and again on 2026-11-09. Parts A-E in release order, so the tail can be reconsidered (§41.4 of the tech spec). v1.5: the hook title moved to V6.2 |
| Background music with automatic speech ducking (the person's own track, v1.5), and sound effects: **(v1.5) synthesized whooshes, pops, clicks and risers placed by rules on every visual entry; voice tuning by measured need** | P0 (V6.7) | Founder decisions 2026-10-21 and 2026-11-09 (§12.16) |
| **Layers:** B-roll from the user's own clips and from **Pixabay stock** (v1.4), placed by the plan, **by hand on a word** (v1.4) or by chat; transitions at joins; screen-and-face and full-frame layouts | P0 (V6.8) | §12.12. Founder decision 2026-11-02: own files only; reversed 2026-11-08: own files and Pixabay. (v1.5) Zoom transitions at joins; the quality check (§12.16) |
| **Founding release** (live payments, the founding offer) after a blind test against Submagic (E-16) | After V6.8 (v1.5; v1.3: after V6.6) | Founder, 2026-11-09: sell when every element of the benchmark exists |
| **Chat editing** | P0 (V6.9) | §12.11. No timeline ever; ask in words |
| Brand kit: logo, colours and fonts applied to every template (one kit per account; Creator plan). **Creator memory. Back-catalogue queue (Creator)** | P0 (V6.10) | §12.14, §12.13. Why people stay subscribed (founder) |
| **Person mask:** text behind the speaker, background change, each gated on quality | P0 (V6.11), gated | §12.15. Founder decision 2026-11-02: both, at launch, if TE-25 passes; either can stay off |
| **Quality pass** against the founder's benchmark (Submagic-made shorts and caption references, kept outside the public repository) | P0, after V6.11 (v1.4) | Founder, 2026-11-08: build every feature first, then inspect and improve against the benchmark (`buildplan.md` §8A.12) |
| Safari/Firefox support | P1 | Promoted when E-8 shows > 25% of visitors blocked by browser |
| Mobile browsers | P1 | Same trigger as above plus a viable WebGPU/WebCodecs path |
| Pro plan ($29) | P1 | Promoted when two P1 features exist that are worth a higher tier (§21) |
| Pause-subscription option | P1 | Promoted when churn reasons cite "stopped posting" > 30% |
| Cloud transcription (signed-in default, the card's choice) | **Built** (cloud transcription release) | The founder brought it forward on 2026-11-06: about 13× faster than the device on a minute of speech (TE-C4) |
| Three versions per recording, remix | P2 | After retention evidence (§21); a second plan request with a different "energy" setting is the cheap path |
| Style DNA | P2 | After retention evidence |
| Local LLM producer of the edit plan | P2 | When provider cost or terms (TE-15) or the privacy messaging (E-6) argue for it |
| Stock-footage B-roll suggested by the plan | **P0 (V6.8), v1.4** | Moved into launch scope by the founder on 2026-11-08 (Pixabay); motion-graphic B-roll stays P2, §21 item 14 |
| Chapter markers, lower third | P2 | §21 item 15 |

**V6.x scope rule (v1.3).** The eleven V6.x rows are launch scope in release order. **V6 (accounts and checkout) comes first.** If V6.x overruns, the founder chooses between moving the launch date and cutting from the tail; a release is never shipped half-done, and V7-V10 keep their five weeks (`buildplan.md` §8A). At any release's stop-and-ask trigger (`technicalspec.md` §41.4), the founder decides to keep, defer or drop.
| Native desktop app | P2 | After web product validates |
| Multi-language | P2 | After English retention validates |

### 9.6 Engineering scope (what must be built underneath)

This list may be large; that is acceptable. It exists to make the P0 capabilities genuinely good.

- **Rust core** compiled to WASM: transcript model, detector, scene graph, animation engine, deterministic frame generation from (source, transcript, style, edits, renderer version).
- **Workers:** media decode/probe worker, ASR worker, audio DSP worker, render/encode worker, with a typed message protocol, cancellation and progress.
- **Local ASR:** quantized speech model via an ONNX- or Rust-based runtime, with WebGPU or WASM-SIMD paths, word-level timestamps, sentence segmentation, number-word normalization ("ten thousand" to 10,000).
- **Model manager:** versioned download from CDN, resumable, hash verification, cache in browser storage, memory budget, and unload.
- **Audio chain:** decode, resample, denoise, high-pass, compressor, limiter, loudness measurement and normalization. The chain changes how the audio sounds, never how long it is. **Program audio (v1.2):** a separate step applies the plan's cuts (snapped to silence, cross-faded), mixes the music track and ducks it under speech, and produces the audio the preview plays and the export encodes.
- **Edit plan (v1.2):** a capability catalog that lists everything the renderer can draw (with parameter types and limits) and generates the LLM prompt and output schema from code; a server route that forwards the transcript to the LLM and returns the plan; a validator that drops any item that points at missing words, exceeds a limit or would show a number not spoken; the rule detector as the second, offline producer of the same plan.
- **Long path (v1.3):** a source type and route beside the clip path; windowed audio on disk; checkpointed transcription; a Director request and validator working at sentence level; an `Assembly` that makes each short a virtual clip, so the existing pipeline runs unchanged; local face detection and shot analysis; typed chat operations.
- **Renderer:** Vello scene per frame (captions, events, furniture, cards, brand logo, watermark) with the video itself as a scene node (transform, zoom, mask, transition), with typography (font registry including brand fonts, shaping, line breaking, safe areas, emoji).
- **Encoder:** WebCodecs H.264 and AAC encode, a Rust MP4 muxer, A/V sync handling for variable-frame-rate sources.
- **Preview:** real-time or near-real-time playback with audio, scrubbing, and low-resolution mode.
- **Persistence:** OPFS/IndexedDB for source references, transcripts, edits and a render cache; recent-clips list; "Delete local data."
- **Backend (thin):** Rust/Axum or equivalent; auth (magic link), session, billing webhooks (idempotent), entitlement tokens, usage receipts, rate limiting, allowlisted analytics ingestion.
- **Observability:** client performance timing per stage (allowlisted, anonymous), error taxonomy, server logs and uptime monitoring.
- **Test harness:** golden-frame image tests for captions/events, a corpus of 40+ real clips (VFR, rotated, noisy, accents), performance regression runs on R1/R2, a network-capture privacy test, and detector precision/recall tests on hand-labeled transcripts.

### 9.7 Engineering complexity budget

| Subsystem | Complexity | Ongoing cost | Why needed | Priority |
|---|---|---|---|---|
| Local ASR + number normalization | Very high | Correctness (accent/noise), device variance, model updates | Captions and events depend on it; H3 | P0 |
| Visual-event detector (rules + confidence) | Medium | Rule tuning, false-positive debugging, regression tests | The differentiator; precision principle | P0 |
| Vello renderer + typography + scene graph | Very high | GPU/driver variance, text layout bugs, WebGPU differences | Output quality is the product | P0 |
| WebCodecs encode + MP4 mux + A/V sync | High | Browser/codec variance, odd inputs (VFR, rotation) | Export is the activation event | P0 |
| Audio DSP chain | High | Tuning, artifacts on odd mics, CPU budget | Perceived quality; J7 | P0 |
| Worker orchestration + cancellation + progress | High | Race conditions, memory pressure | Prevents UI freezes on long jobs | P0 |
| Preview player (synced) | High | Sync bugs, performance | The AHA moment | P0 |
| Model manager (download/cache/verify) | Medium | Cache invalidation, storage quotas | First-run time-to-value | P0 |
| Capability detection | Low | Needs updating with browser changes | Honest reach; E-8 | P0 |
| Thin backend (auth, billing webhooks, entitlements, receipts) | Medium | Security, webhook idempotency, support | Revenue and free-tier limits; H2 | P0 |
| Analytics + privacy network test | Low-Med | Test maintenance | Measures activation; proves privacy | P0 |
| Persistence (OPFS/IndexedDB) | Medium | Storage quotas, migrations | Recover from refresh, recent clips | P0 |
| Test corpus + perf harness | Medium | Maintenance as features grow | Core Philosophy bottleneck 7 | P0 |
| Edit plan: catalog, prompt, server route, validator, fallback | High | Provider terms and free-tier limits (TE-15), prompt upkeep, hallucination handling, latency | The v1.2 product: every V6.x feature reads it | P0 (V6.2) |
| Cuts and time map (program timeline) | High | A/V sync across cuts, click-free splices, every timestamp mapped once | Final-video quality (founder) | P0 (V6.3) |
| Music mix + ducking | Medium | Mix bugs, loudness of the mix, licensing UX (TE-16) | Final-video quality | P0 (V6.7) |
| Cards, furniture, transitions, kinetic presets, video masks | High | Golden-frame upkeep per item kind, layout bugs | The Submagic-class look | P0 (V6.6b) |
| Brand kit (logo, colours, fonts) | Medium | User font files, logo images, theme coverage of every template | Retention | P0 (V6.10) |
| Long source, windowed audio, resume | High | Big-file variety, disk, long jobs dying | The long path | P0 (V6.1) |
| Assembly and rebased transcripts | Medium | Decoder jumps, sync across joins | Stitching, without touching the clip path | P0 (V6.3) |
| Director: prompt, route, validator, rules fallback | High | Judgement quality; prompt upkeep | The product of the long path (E-13) | P0 (V6.4) |
| Face-tracking reframe and shot signals | High | Model, failure cases | Long landscape videos | P0 (V6.5) |
| Following the speaker (v1.5) | High | Wrong-person switches, small or turned faces (TE-27) | Two-person podcasts | P0 (V6.5) |
| Automatic colour and light, blur (v1.5) | High | Doing harm on good footage (TE-29), GPU budget | The picture looks graded | P0 (V6.6a) |
| Motion by rules, sound effects by rules, Enhance switches, quality check (v1.5) | Medium | Tuning against the benchmark; golden upkeep | The worst short is good, not only the best | P0 (V6.2, V6.7, V6.8) |
| Multi-layer compositor, asset box | High | Decoder count, memory, golden upkeep | B-roll, joins, layouts | P0 (V6.8) |
| Chat: ops, route, panel | Medium | Prompt upkeep; refusals | No timeline, ever | P0 (V6.9) |
| Creator memory, queue | Medium | Long-running jobs, storage | Retention | P0 (V6.10) |
| Person mask | High | Model, flicker, licence | Demos that spread; gated (TE-25) | P0 (V6.11), gated |
| Cloud transcription | High | Consent, security, cost, privacy-claim integrity | Everyone signed in | **Built** (cloud transcription release): the card before every drop, P-1 revised, Zero Data Retention, a global budget under the free tier |
| Local LLM producer of the plan | Very high | Download size, device variance | Offline plan, no provider dependency | P2 |
| Native desktop build | High | Second platform to ship and support | Power users | P2 |
| Style DNA | High | Analysis models, UX | Retention/stickiness | P2 |

### 9.8 Hypothesis-to-engineering map

| P0 component | Hypothesis it tests/enables | If removed | Simpler alternative: decision |
|---|---|---|---|
| Local ASR with word timestamps | H1, H3 (and H4: audio stays local) | No captions, no events; product impossible | Cloud ASR: rejected for MVP because it breaks H4 and makes per-export cost non-zero (H2 pricing); retained as P1 consented fallback |
| Rule-based detector | H1 | No offline plan: with the AI plan off or unreachable, no visuals at all | Kept in v1.2 as the offline producer of the edit plan; manual event insertion by user: rejected (e) (defeats the "speech is the interface" claim) |
| AI edit plan (v1.2) | H1 | Back to the rules' four event kinds and no cuts: a caption tool, not an editor | Local LLM: P2 (download size, device variance); rules only: kept as the fallback |
| Long path and Director (v1.3) | H5 | Offcut stays a short-clip tool, with one fewer reason to pay | YouTube links and server processing: rejected, it breaks H4 and the cost model |
| Vello renderer | H1 (visual quality) | Visuals fall back to generic caption look | Canvas2D overlay: **accepted as pre-agreed fallback** if Vello/WebGPU readback fails E-4 (§20.4) |
| WebCodecs + Rust muxer | H3, activation event | No export | FFmpeg.wasm: rejected for MVP (f)(c) (larger, slower single-thread builds, licensing review); revisit only if codec coverage blocks launch |
| Audio chain | H1 (post-ready output) | Output sounds unprofessional; weaker AHA | Loudness normalization only: accepted as fallback if denoise exceeds the CPU budget |
| Preview player | H1 (AHA moment) | User sees nothing until export; activation drops | Render a low-res preview video: accepted if real-time preview misses budget |
| Review UI (word edit, event toggles) | H1 | Any ASR/event mistake is a dead end | None; this is already minimal |
| Model manager | H3 | First run unreliable; repeat runs re-download | Re-download every session: rejected (e) (breaks time-to-value) |
| Thin backend + entitlements | H2 | No payment, no free-tier limits | Payment link only: rejected (b) (no entitlement, no usage counts, manual support) |
| Privacy network test + analytics allowlist | H4 and trust | Privacy claim unverifiable; measurement impossible | Policy page only: rejected (c) (claims without proof are a legal and trust risk) |
| Test corpus + perf harness | H3 and stabilization | Odd inputs break launch week | Manual testing: rejected (b) (burden grows with every release) |

### 9.9 Effort estimate

**Scope by stage (lines of code shown only as a rough size reference; per §0 they do not drive decisions, and I do not use the source document's 130k-230k estimate).**

| Stage | What it is | Rough size (ASSUMPTION) |
|---|---|---|
| Proof of concept | One clip end to end: ASR → captions → export, one event type, hard-coded style | 8k-12k LOC |
| Functional MVP | All P0 features working on dev machines | 25k-40k LOC incl. tests |
| Hardened MVP | P0 on R1/R2 with corpus, error handling, privacy test, billing | 35k-55k LOC incl. tests |
| Mature product | P1 features, more events, Safari/Firefox, music | 80k-120k LOC |
| Full competitor (CapCut-class editor) | Timeline, effects, templates, mobile, team | 300k+ LOC, multiple engineers |

**Time estimates for this founder (A-1, A-2)**

| Estimate | Duration | Basis |
|---|---|---|
| **Implementation time** (producing the software to functional MVP, M1) | **5 weeks** (weeks 1-5) | Founder estimates about 20k working LOC per week with LLM assistance; 25-40k LOC implies 1.5-2 weeks of raw generation, but WASM ASR, Vello-in-browser and WebCodecs integration each need spikes and debugging, so I budget 2.5-3× raw generation time |
| **Stabilization time** (reliable for real users: odd inputs, device matrix, performance, security, billing) | **5 weeks** (weeks 6-10) | Real-world clips (VFR, rotation metadata, noisy audio, accents) and R1/R2 performance tuning are emergent; budget equals implementation because debugging a large generated codebase is the known bottleneck |
| **Validation time** (learning if users want and pay) | **26 weeks** after launch (decision at Month 3 gate; final at Month 6), with early experiments in Month 0 | Distribution is slow and non-linear; one conversion experiment needs weeks of traffic |

**Validation time is usually the longest and is the one that decides whether the business works.** Month 0 (implementation + stabilization) is 10 weeks; validation to the final decision is 26 weeks after that.

**Milestones**

| ID | Milestone | Date (A-2) | Gate |
|---|---|---|---|
| M0 | Feasibility + demand gate | End of week 3 (Sun 1 Nov 2026) | E-1, E-3, E-4 results; stop or continue the build (§20.4) |
| M1 | Functional MVP | End of week 5 (Sun 15 Nov 2026) | All P0 working on dev machines |
| M2 | Hardened MVP + launch gate | End of week 10 (Sun 20 Dec 2026) | §20.1 checklist complete |
| M3 | Month-3 decision | Sun 21 Mar 2027 | §20.5 decision rule |
| M4 | Month-6 decision | Sat 19 Jun 2027 | §20.5 decision rule |

*v1.3: dates above are as first written. M2 moves with V6.11; M3 and M4 stay 13 and 26 weeks after launch (§13.1). V1-V5 are closed and live.*

---

## 10. Core User Journey (first 10 minutes)

**Activation event:** first successful export (J11). **AHA moment:** the first playback of the preview showing the user's own clip with animated captions and at least one visual event (J7), which happens **before** sign-in.

| Step | What the user sees and does | Notes (limits and microcopy) |
|---|---|---|
| **J1** Land | Hero (design release, 2026-11-05): "Say it once. Post it finished.", with the hero clip, "Start free" into the app and "See how it works". Below: "Your video stays on your computer." and a link to "What leaves your device." Then short product clips (made with Remotion, in place of the hand-made demo), every feature that works today, everything after V6 marked "Coming soon", the drop zone, pricing and the update form | Supported-browser line: "Works in Chrome or Edge on Windows and Mac." Shown before any upload |
| **J2** Drop | Large drop zone: "Drop a clip (up to 90 seconds, English)." Optional "Try with a sample clip". (v1.4, with the long path on: one link, "Got a long video? Make shorts from it", to its own page, L0. v1.3's single drop zone for both is replaced.) | No account needed for a clip |
| **J3** Capability check | 1-3 second check, green tick or a clear unsupported page (§9.3) | Tone: plain, specific. "Your browser can do this." |
| **J4** First-run model download | "One-time setup: downloading the speech model (about 250 MB). It stays in your browser, and your video is not uploaded." Progress bar with time remaining. **Only on the device path:** a cloud clip needs no model (cloud transcription release) | TARGET ≤ 90 s at 25 Mbps (§20.2). Subsequent runs skip this |
| **J5** Validate | Probe the file; reject with specific guidance if constraints fail (§9.4) | Counted as `reject_reason` only |
| **J6** Processing | Live "what we found" feed: "Transcribing…", "Planning your edit…", "Found: 3-item list", "Found: $2k to $20k", "Trimmed 6.4 s of pauses", "Cleaning voice" | Shows real plan items and cuts only; no fake steps. If the plan falls back to rules, the feed says so once ("Edit plan unavailable: using local rules") |
| **J7** Preview (AHA) | Preview plays with animated captions and visual events; two buttons: "Looks good → Export" and "Fix something" | If no events were confident enough: "No numbers or lists found in this clip. Captions and cleanup applied." (honest, no padding) |
| **J8** Review | Edit caption words inline; toggle each plan item on/off; restore any cut (or switch on filler-word cuts); music on/off and track choice; choose a caption preset; choose the brand kit; set crop offset for landscape; add B-roll on a word (V6.8, v1.4); copy the hook title, post caption and hashtags; "Regenerate plan" (counts against the plan quota) | No sliders beyond the crop offset and the B-roll part picker (principle 3, v1.4). No timeline or keyframes. Cuts are only what the plan proposed; the user can restore, not invent, cuts (principle 7) |
| **J9** Export click | If not signed in: "Create a free account to export. Free: 3 exports a month, 720p, small watermark. Creator ($15/month): no watermark, 1080p." Magic-link email sign-in | **Limits shown before the click that triggers them, never as a surprise.** Export counter ("2 of 3 left") visible on the button |
| **J10** Render + export | Progress by stage with elapsed time; "Everything is rendering on your computer." | TARGET total for 60 s clip ≤ 3 minutes on R1 (§20.2) |
| **J11** Download + summary | MP4 downloads. "What we changed": voice cleaned, N pauses shortened (T seconds removed), music added, N words emphasized, M visual moments, chapters (only changes that were made) | **Activation event fires here** |
| **J12** Upgrade / return | On a free export: "Remove the watermark and export in 1080p: Creator, $15/month." Also: "Make another clip" | One prompt, dismissible; no email blast on first visit |

### 10.1 The long-path journey (v1.3)

**Activation:** first exported short from a long video. **AHA:** the list of shorts, then the first one playing with captions and framing.

| Step | What the user sees and does | Notes (limits and microcopy) |
|---|---|---|
| **L0** Page (v1.4) | "Make shorts from a long video": its own page, from the landing page and from J2's link. Drop zone: "Drop a video (up to 45 minutes, English, one or two people on screen)." A file of 90 seconds or less dropped here opens as a clip | The person chooses the path; duration does not |
| **L1** Gate | After the probe, before any work: "This is a long video. It needs a free account, and its sound is sent to be transcribed; the video stays on your computer. Free: 1 long video a month. Creator: unlimited." Sign in, or cancel. If cloud transcription is off right now, the page says so and offers to try later | Limits and what is sent shown before the work (principles 4 and 5). A clip never reaches this |
| **L2** Disk check | If space is short for the extracted sound and the chosen parts: the exact need, and the option to clear older clips | No surprise failure mid-job |
| **L3** Working | Stages with progress and an honest time left: sound, looking at shots, transcribing, choosing shorts, keeping the chosen parts, preparing. "You can use your computer. Keep this tab open." Pause and Cancel. Progress also in the tab title. If transcription cannot continue (offline, busy, signed out), the job pauses and says why, with Continue | Wake lock on. Live feed shows real findings only |
| **L4** Return | After a reload, crash or sleep: "Continue your video" from the last saved step. Transcribing, choosing and preparing continue at once; a step that reads the video asks "Drop the same file to continue" | At most the windows in flight are lost |
| **L5** List | Shorts appear as each is ready: poster, title, length, parts count and one-line reason. Keep, delete or open | First short can be opened while the rest finish |
| **L6** Open a short | The clip review (J7, J8) with the plan already applied, plus the list of parts with their words, "Back to all shorts", B-roll by hand from V6.8 and chat from V6.9. The original file is no longer needed | Same editor; nothing new to learn |
| **L7** Export | "Export kept shorts": the count and exports left, before the click | Free: each short is one of the month's 3 exports; the rest stay in preview. Sequential, on this computer |
| **L8** Summary | What changed per short; post captions and hashtags to copy | Activation fires at the first short |

**(V6.1, as built; dark in production.)** L0 to L4 exist behind `long_video`, and V6.1 ends at the master transcript: "Your video is transcribed", its length and its sentences at their times, and "Shorts from it come next." (V6.4 replaces that screen with L5). As built: the gate (L1) asks in order for an account, then whether transcription is on ("Checking that transcription is available…", at most 45 s), then the disk, then says what is sent and offers Start; no limit line yet (the Free plan's one long video a month comes with V6.4). L2 shows the amount needed but no way to clear older clips (V7). L3 has two stages, "Reading the sound" and "Transcribing N of M", each with its bar, the time left once a tenth is done, and Cancel; there is no user Pause button (the job pauses itself with a reason: off right now, busy, signed out, offline, or the computer slept). L4 is as written: transcription continues without the file; the sound step asks for the same file and refuses another. The landing page is unchanged in V6.1 (D-425).

---

## 11. Monetization

**Why the cost structure allows this pricing.** Real cost of goods per export is near zero because compute runs on the user's machine (the model is downloaded once from a CDN; static asset egress is the only per-user variable cost, and object storage with free egress can host it: *general-market knowledge, re-verify*). Real recurring costs are: merchant-of-record fees (about 5% + $0.50 per transaction, A-9, taken out of each sale, never paid up front) and the founder's support time (the real scarce resource). Fixed infrastructure is $0/month (A-10): everything runs on free tiers.

| Plan | Price | Included at MVP | Adds later (P1) |
|---|---|---|---|
| **Free** | $0 | Full-length processing and preview; 3 exports/month (the quota resets on the UTC calendar month); 720×1280; small watermark; every caption preset, plan item, cut and music feature; AI edit plans up to the free plan quota (§12.8); **1 long video a month, with all its shorts in preview (v1.3)**; 20 chat messages a month | Nothing planned; free stays this usable |
| **Creator** | $15/month or $144/year ($12/month equivalent, 20% off) | Unlimited exports (soft fair-use ceiling 100/month: exports above it are logged, never blocked); 1080×1920; no watermark; everything in Free; the larger AI edit plan quota; **unlimited long videos** (soft ceiling 200 a month, logged, blocked at 3×) **and the back-catalogue queue; chat** (soft 1,000 a month); **brand kit** (V6.10) | |
| **Pro** (not at MVP) | $29/month (planned) | Not sold at MVP | Three versions per recording; brand presets; priority support (P1, §21) |

**On the landing page (design release, 2026-11-05).** The prices above are unchanged. Free has a "Start free" link into the app. Until the founding release turns on live payments (`technicalspec.md`, v6implementation D-303), Creator shows its price and "On sale soon", with no way to buy on the page.

**Pricing metric and why.** Flat monthly per-creator subscription with effectively unlimited exports, not credits or minutes. Reason: competitors reportedly meter by credits or minutes, which creates friction; our marginal cost is near zero, so metering would be artificial. The Free tier limit (3 exports) exists as a conversion mechanism, not as cost control.

**The one real per-clip cost (v1.2).** Each AI edit plan is one LLM request over a transcript of a few thousand tokens. It is the only cost that grows with use, so it is the only quota enforced on the server (`technicalspec.md` §40.8). The quotas are set by TE-15 from the chosen provider's free-tier limits and are meant to be generous beside credit-metered competitors ("bigger quotas"): a clip with no plan left still gets the local-rules plan, so a spent quota never blocks an export. Starting values (ASSUMPTION, TE-15): anonymous 3 plans per day per network, Free 40 per month, Creator 1,000 per month soft (logged above, blocked only at 3× to stop abuse). v1.3: a paid provider is allowed under the cap (A-3); raising the cap is the founder's call.

**Long videos and chat (v1.3; v1.4).** A long video costs one Director request plus one plan per short, and (v1.4) its cloud transcription; a chat message is one request. **The founder allows a paid provider with a monthly cap** (A-3). Estimated cost of a 45-minute video with 8 shorts: about 0.9 to 4 cents of AI on cheap models at rates seen on 2026-11-01 (third-party listings, re-verify, TE-15), plus about 3 cents of transcription on Groq's paid tier ($0.04 per hour of audio, `technicalspec.md` §52), and model choice follows quality, since clip choice is the product: 10× that would still fit. Groq's free tier carries only about two 45-minute videos an hour across all users, so its paid tier starts before the private alpha (founder's decision of 2026-11-06, brought forward). At 3,000 paying users making 8 long videos a month at 30 cents each, the bill is about $7,200 a month against about $42,750 of revenue; at 3 cents it is $720. **At the cap,** AI requests stop until next month and the local rules take over: fewer, simpler shorts, marked "Picked without AI", and nothing blocks. **Creator stays $15/month** (H2, E-5 reads it). Long-video and chat quotas are server-enforced because they are real costs; export limits stay conversion mechanisms.

**Honest statement about enforcement.** Because processing is local, limits are **conversion mechanisms, not security controls**. A determined user can bypass the watermark or export counter by tampering with the client. That is acceptable: the target user pays for convenience and updates, and the cost of enforcing harder (obfuscation, server-side rendering) breaks the model. Metering at a high level: the client holds a signed entitlement token (plan, period, free exports remaining) cached for up to 7 days offline; each export sends an idempotent usage receipt (`export_id`, no content) to the server; the server computes counts; the watermark is rendered into the video by the pipeline unless the token says Creator.

**Upgrade trigger design.** (1) Visible export counter before every export. (2) After the first watermarked export, one dismissible prompt framed as removing the watermark and getting 1080p. (As built in V6, read with J12's "On a free export": after each watermarked export, once, until it is dismissed.) (3) At export #3 on Free, "this is your last free export this month" before the click. (4) No upgrade popups during processing or editing.

**Target plan mix.** At MVP, 100% of paid users are on Creator. Billing mix (ASSUMPTION): 75% monthly / 25% annual in the base case, giving a blended revenue per paid user of 0.75 × $15 + 0.25 × $12 = **$14.25/month** (§13.2). Pro is excluded from all numbers in this document.

---

## 12. Flagship / Trust-Critical Feature Deep Dive: Speech-to-Visual-Events Pipeline

**Why this feature.** It is the product's promise ("what you say becomes what you see") and the greatest risk to it: a wrong or ugly event damages trust faster than any missing feature, and the on-device claim is verified here.

### 12.1 What ships (P0)

Local transcription with word timestamps → number and structure detection → events rendered through Vello over the video. **v1.2:** detection becomes one of two producers of the edit plan (§12.8); the AI plan is the default producer from V6.2.

### 12.2 Variants and templates (MVP)

| Event | Trigger examples | What the viewer sees |
|---|---|---|
| **NumberReveal** | "$10k last month", "800 milliseconds", "ten thousand events" | The number appears large with a count-up or scale-in on its word timing; unit and label small beneath |
| **ListReveal** | "three reasons", "first… second… third…" | A count header ("3 REASONS") then each item appears at its spoken timestamp, numbered |
| **FromTo** | "from $2k to $20k", "800 ms down to 120 ms" | Two values side by side with an arrow and proportional bars; the second value emphasized |
| **KeywordPop** | Acoustically and lexically emphasized words (energy/pitch above sentence baseline, or a recognized emphasis word) | The word scales and highlights in the caption |

**Caption styles:** Clean, Bold, Mono (typography, color, motion curves differ; same layout rules). Default: Clean, the premium one since V4: the spoken word sits on a coloured pill that slides from word to word. Mono is the style first called Tech (its stored id is still `tech`). In every style the captions break at natural phrase boundaries, each stays on screen long enough to read, and numbers can be shown in the accent colour.

### 12.3 Configurable at MVP vs later

At MVP (V4-V6): style choice (3), visual on/off per visual, crop offset (by dragging the video too), caption word edits (with "Change all" for a word heard several times), caption position (dragged on the preview, with Top, Middle and Bottom snaps), numbers highlighted on or off, undo and redo. V6.1-V6.11: plan item on/off, cut restore, filler cuts on/off, music on/off and track, caption presets, brand kit, regenerate plan, keep or delete a short, remove or restore a part, assets, chat. Not configurable at launch: per-item timing, the visuals' positions, animation curves (no timeline, principle 3).

### 12.4 Why this technical approach (and what was rejected)

| Approach | Decision |
|---|---|
| Caption templates only (what competitors do) | Rejected: does not test H1 or differentiate |
| Rule-based detector over word-timestamped text with number parsing | **Chosen for V3-V6, kept as the fallback producer in v1.2**: deterministic, debuggable, small, testable with a labeled corpus; precision tunable by threshold. Ceiling: it cannot tell an intentional pause, an implied list or a topic shift |
| Local LLM classification | Rejected (f): hundreds of MB to GB of model weights, variable performance; P2 as an offline producer of the same plan |
| Cloud LLM edit plan over transcript text, timings and prosody scores | **Chosen from V6.2 (v1.2)**: understands context the rules cannot; nondeterminism is contained by storing the plan per clip and validating it against the transcript; media stays local (§12.8) |
| Hand-authored event insertion by the user | Rejected (e): defeats "speech is the interface" |

### 12.5 Timing / data model

- `Word { text, start_ms, end_ms, confidence }`
- `Sentence { words[], start_ms, end_ms }`
- `DetectedEvent { kind, span: (start_ms, end_ms), params, confidence, enabled }` where `params` is typed per kind (e.g. `FromTo { from: Value, to: Value, label? }`)
- Events with confidence below the tuned threshold are not created. Each event keeps the anchor word indexes so edits to caption text re-run detection for that sentence.
- Rendering is deterministic: the same (source, transcript, events, style, renderer version) yields the same frames, which makes golden-frame testing possible.
- Two timelines from V4 (v1.2): **recording time** (when a word was spoken in the source) and **program time** (when it plays in the export). A time map built from the plan's cuts converts one to the other; transcript and plan keep recording time, the scene, preview and export run on program time. Until a plan contains cuts (V6.3) the map is the identity, so program time equals recording time and the export is as long as the source clip, pauses included.

### 12.6 Cloud transcription and its consent rules (built in the cloud transcription release)

**As built (founder, 2026-11-06; `docs/cloud_transcription/`).** Cloud transcription is the default for a signed-in person and is chosen in a card inside the drop zone, before any clip is dropped: "Cloud · fast" or "On device · private", with one line saying what each means ("Only your clip's sound is sent to write them, and it is not kept. Your video stays on your computer."). The choice is remembered on the device. Signed out, everything runs on the device and the card offers sign-in. The provider is Groq (`whisper-large-v3-turbo`), with Zero Data Retention on; the sound is compressed to Opus and passes through our server in memory only. Any failure finishes the clip on the device with one notice. **v1.4: a long video has no card and no device path.** Its sound is always transcribed in the cloud, in windows of at most 90 seconds; the long page says so before the work (L1), and a failure pauses the job with Continue instead (§12.9). This replaces the per-clip prompt below, and the per-export mode display: the card is shown before every drop, and the processing card says where the captions are written. The settings table and the Privacy Policy name the provider. Video frames are never uploaded in any mode. The founder approved the wording; a lawyer's review is on the launch checklist (§17).

**As first written (superseded):**

This section is about uploading **audio** for transcription. The v1.2 edit plan sends transcript text only and has its own rules (§12.8). If E-3 shows too many users miss the time budget, offer **"Faster processing (uses our servers)"**, off by default. Consent rules: a per-clip explicit prompt (not a persistent toggle) saying what is sent (audio only, for transcription), where it goes, that it is deleted after processing (retention period to be set by the tech spec), and that the "never uploaded" claim does not apply to that clip. The settings page shows the mode used for each export. Video frames are never uploaded in any mode. Legal review (§17) is required before launch of this feature.

### 12.7 What the settings/account page must show

A "What leaves your device" table, always visible:

| Data | Leaves your device? |
|---|---|
| Video, frames, music files, assets, brand logo and fonts, captions as edited, project edits | **No** |
| A clip's sound, compressed (cloud transcription release) | **Yes, while "Cloud" is chosen in the drop zone and you are signed in:** to our server, which sends it to the transcription provider named here and returns the words. Neither keeps the sound or the words after the request. Choose "On device" and nothing of the clip leaves |
| A long video's sound, compressed, in pieces of at most 90 seconds (v1.4, V6.1) | **Yes, always, for every long video:** the same way and the same rules as a clip's. The long path has no on-device choice, and the long page says so before you start |
| Words you search for stock B-roll, typed or chosen by the plan (v1.4, V6.8) | **Yes, while "Stock B-roll (Pixabay)" is on:** to our server, which asks Pixabay and returns clips. We do not keep or log the words; the results are kept for a day under a scrambled key, as Pixabay requires. The stock clip you choose comes straight from Pixabay's servers to your browser |
| Transcript text, word timings and per-word loudness/pitch scores (v1.2, from V6.2) | **Yes, while "AI edit plan" is on:** once per plan request, to our server, which forwards it to the AI provider named here and returns the plan. Neither we nor the provider keep it after the request (provider terms recorded by TE-15). Switch it off and nothing of the clip leaves |
| Sentence text and timings of a long video, and shot categories like "talking head, face left" (v1.3, V6.4) | **Yes, while "AI edit plan" is on:** once per long video, to choose shorts. Same rules as above |
| What you type in chat, with that short's words (v1.3, V6.9) | **Yes, while "AI edit plan" is on:** per message. Same rules. Never used in analytics |
| Labels and notes you type on your assets (v1.3, V6.8) | **Yes, while "AI edit plan" is on:** in plan and chat requests, so it can place them. The assets themselves do not leave |
| Creator memory: counts of which kinds of item and short you keep (v1.3, V6.10) | **Yes, while it is on:** as numbers in AI requests. No text. Switch it off or reset it in settings |
| Your email and plan status | Yes, to our servers |
| Count of exports and of plan requests (no content) | Yes |
| Anonymous usage events from an allowlist (e.g., stage timings, error codes) | Yes, and the list is shown on the page |

It also shows: processing mode ("Local rendering; AI edit plan on/off" with the switch), the AI provider's name and data-use summary, model version and cache size with a "Clear local data" button, and a short "Verify it yourself" note (open your browser's network panel during processing and confirm that no media is sent, and that the transcript goes out only in the one plan request).

### 12.8 The AI edit plan (v1.2, from V6.2)

**What it is.** One request per clip (and per "Regenerate plan"). The client sends the transcript with timings and prosody scores; the server adds a system prompt that lists everything the renderer can draw and forwards both to the LLM; the LLM returns a plan in a fixed JSON schema; the client validates it and the renderer draws it. **The editor applies the plan automatically (v1.4, founder 2026-11-08):** there is no approve-each-step screen; the person sees a finished short and switches off or undoes what they do not want. The same flow serves both paths. The plan is stored with the clip, so preview, export and a reload all use the same plan (deterministic rendering, §12.5).

**What the plan contains.**

| Part | Examples |
|---|---|
| Cuts | Lead-in and tail silence, shortened pauses, intentional pauses marked to keep, filler words and false starts (off by default) (§9.2.1) |
| Emphasis picks | Which words get colour, an icon pop or a zoom: a snap onto one strong word or a slow push across a sentence that builds (auto zoom, v1.4); (v1.5) which word becomes a punch word at twice the size |
| Structure visuals | List builder (items appear as each is spoken), number moments (counting number or simple bar), quote card, definition card, X-versus-Y comparison, step tracker, timeline, checklist |
| Emoji | Transcript-aware emoji beside a word or (v1.5) under the keyword, from the catalog's fixed emoji set |
| Structure of the talk | Hook (opening line), topic shifts (each gets a transition), call to action |
| Furniture | Hook title (v1.5: a lockup over the first 2 seconds, from V6.2), progress bar, end card with a call to action |
| Music cue | Whether music fits and its energy (the user picks the track) |
| B-roll (V6.8) | Which words get one of the person's own clips, or a stock search on Pixabay (v1.4) |
| Text extras | Hook title, post caption, hashtags (shown in review to copy; not drawn unless an item uses them) |

**How it is kept honest.**
- The **capability catalog** in code lists every item the renderer supports with its parameters and limits. The prompt and the output schema are generated from it, so the model can only ask for what exists, and a new effect is one catalog entry plus its drawing code.
- The model **never writes a number**. A number moment, a comparison or a bar points at the words that said the number; the value shown is parsed from the transcript, including after the user corrects a mis-heard word. Free text the model writes (titles, list labels, chapter names) has length limits and may contain no digits that the transcript does not contain.
- Every item points at real words. An item outside the transcript, over a limit, overlapping a cut, or of an unknown kind is **dropped**; the rest of the plan is kept.
- If the request fails, times out, is over quota, or the switch is off, the **local rules** make the plan (V3's four kinds, no cuts). A clip is never blocked by the plan service.

**Quotas.** The plan request is the one per-clip cost (§11); quotas are enforced on the server and set by TE-15.

**Measured by.** E-11: plan items kept versus switched off in review, cuts restored, and the "Would you post this?" answer for AI plans versus rule plans (§19).

### 12.9 From a long video to finished shorts (v1.3, V6.1, V6.3, V6.4; v1.4)

**Its own page (v1.4).** "Make shorts from a long video" is a separate page from "Edit a clip" (L0); both open the same editor.

**What it is.** The video, up to 45 minutes, is read where it is on the computer and never copied whole. Its sound is transcribed in the cloud in pieces of at most 90 seconds, each saved as it returns, so a reload loses almost nothing; if transcription cannot continue, the job pauses with Continue rather than switching to the much slower on-device model. A Director request then reads the whole transcript sentence by sentence and returns 5-10 shorts. Each short is 15-90 seconds (aim 30-60), made of **whole sentences**, in up to 4 parts from anywhere in the video, with a title, a style and a one-line reason.

**Rules the validator enforces.**

| Rule | Why |
|---|---|
| Parts are whole sentences, by type | Mid-sentence cuts cannot happen. This is the most common complaint about clip tools |
| 15-90 seconds; at most 4 parts; parts do not overlap | Shorts stay watchable; nothing plays twice |
| Two shorts overlap by at most 20% | No near-duplicates |
| Titles and reasons follow the digit rule (§12.8) | No invented number, even in a title |

**What the prompt asks.** Moments that stand alone for someone who never saw the video. A hook in the first sentence, one idea, a payoff at the end. Join moments only when that makes one complete point (a claim from minute 3 with its proof from minute 8). Never join in a way that changes meaning or creates a claim the speaker did not make. Skip intros, sponsor reads and "like and subscribe". Move a hook to the front only if it means the same there.

**How it is kept honest.** Each short lists its parts with their words, so the creator can see exactly what was joined, remove a part or restore it. If AI is unavailable, the local rules give up to 5 single-run shorts, marked "Picked without AI", so a long job never dead-ends. A Free user out of long videos is told before the work starts.

**After selection,** only the chosen parts of the video are kept on the computer, so each short works without the original file. Each short is then a clip like any other: its own transcript, clean audio at one loudness, its own edit plan applied automatically, cuts, captions and visuals. The two-step flow is the founder's: first the AI says which parts to cut and stitch, then it plans each short exactly as it plans a dropped clip.

**Encoded only if kept.** Preparing a short does not render it. Previews play live, and only what the user keeps gets encoded.

**Time.** About 15-30 minutes for a 45-minute video on the reference laptop (TARGET, §20.2A), most of it looking at shots and preparing shorts, while the computer stays usable. *(v1.3: 30-60 minutes for 20 minutes, most of it on-device transcription.)*

### 12.10 Vision: local signals and framing (v1.3, V6.5)

Small local work over the video gives facts: where shots change; whether each shot is a talking head, a screen with a face, a screen, or footage; where the face is. These go to the Director as plain text categories, so it can skip a section that relies on what is on screen. **No frame leaves the computer** (founder, 2026-11-02).

A short from a landscape video follows the speaker's face with a steady crop: it holds still until the face nears the edge, moves gently, and resets at each cut, so it looks filmed rather than tracked. The user can still drag it.

**Two people on screen (v1.5, founder 2026-11-09).** Most long videos are two-person podcasts, and a crop on the wrong face looks broken. When a shot shows two people, Offcut finds who is speaking by matching each face's mouth movement to the speech, on the computer, with no extra download. The frame cuts to that person at the start of a sentence, as an editor would, and never pans between them. When it is unsure, or both talk at once, it shows both, one above the other. The Framing panel offers "Follow the speaker" (default), "Show both" and "Show one person". A video that already cuts between cameras needs none of this: each shot has one face. Nothing about faces leaves the computer.

### 12.11 Chat editing (v1.3, V6.9)

A chat panel beside the other tabs. "Make this shorter." "Use a stronger hook." "Add the part where I talk about pricing." "Use my office clip when I say headquarters." "No emoji." Each message is one undo step, and the preview updates at once for anything that does not change which sentences play.

**The model returns typed operations, never a new plan or a timestamp,** and each goes through the same validator as everything else. It cannot show a number the speaker did not say, and it says so plainly when asked. Every button and toggle still works; chat is another way to do the same things. A user may ask it to cut a sentence: this is theirs to decide, and it appears in the cuts list with a restore.

### 12.12 Asset box and B-roll (v1.3, V6.8)

The user drops their own clips and images and labels them ("my office", "product demo"). Their own clips matter most when stock cannot help: a creator explaining black holes wants their own diagram or a lecture clip, not a generic space photo. **Stock footage from Pixabay is in too (v1.4, founder 2026-11-08; v1.3 said no stock at launch).** The plan places B-roll on the words that fit, from the person's clips or a stock search; **the person can also place it by hand:** click a word, "Add B-roll here", pick a clip or search stock, and for a video choose which part plays by dragging a window along its frames. Chat places it on request from V6.9. Fit: full, contained, or picture-in-picture. Speech stays on top. The person's assets never leave the computer; their typed labels go with AI requests so they can be placed; stock search words go to our server and Pixabay (§12.7), and each stock clip is marked "From Pixabay". The Terms say the person must hold the rights to clips they add.

Also in this release: dissolve, whip, slide and (v1.5) zoom transitions at the joins of a stitched short, each with a whoosh; B-roll moves gently like the speaker's shots; a split layout for screen recordings (screen above, face below); a full-frame layout over a blurred copy for footage.

### 12.13 Back-catalogue queue (v1.3, V6.10, Creator)

Drop many videos, up to 50 in a session. They run one after another on this computer, with a running estimate. The videos are not uploaded (their sound is transcribed in the cloud, as for any long video, v1.4) and no minutes are counted; this is the clearest difference from credit-metered tools. A very long queue can meet the daily transcription budget; it then pauses and says so. It needs the tab open and the computer awake, and it says so. After a reload, finished videos stay and the rest are dropped again.

### 12.14 Creator memory (v1.3, V6.10)

Each edit teaches the next plan: which kinds of visual and short the creator keeps or removes, usual length, whether joined shorts survive, and preferred caption style. It is stored in this browser as counts, with no transcript text, and sent as numbers in AI requests. Settings shows it in plain words, with Reset and an off switch. It is the start of the moat (§7).

### 12.15 Person mask (v1.3, V6.11, gated)

**Text behind the speaker** on a hook, and **background change**, for shots with a steady camera and a visible person. A person mask is computed once per short when first used, stored locally, and needs an extra model download.

**The gate.** On 20 real shorts, a founder panel marks each postable or not; at least 16 of 20 must pass, per effect, along with a flicker measure and a time limit (TE-25). An effect that fails stays off at launch and the founder is asked. A short whose mask is low-confidence is not offered the effect, with the reason shown.

### 12.16 The finished look every short gets, and the Enhance panel (v1.5, the founder's decisions of 2026-11-09)

**The rule.** Offcut survives only if its shorts match or beat Submagic's. The founder's benchmark, Submagic-made shorts read frame by frame on 2026-11-09, showed that the gap is not the captions (Bold is already close) but four things: **something changes about every 2 seconds, every shot moves, sound effects land on each visual entry, and in a podcast the frame follows whoever talks.** Each of these is now done by rules on every short, not left to the AI's choice, so the worst short is good and not only the best. Then Offcut goes past what a server tool does, using the engine on the computer: it corrects each shot's colour and light, and matches the cameras.

| What every short gets | Release | What the person sees |
|---|---|---|
| Motion | V6.2 | Each talking-head shot drifts a few percent in or out, alternating at each cut; the AI's zooms add snaps on strong words. Never on screen recordings |
| Captions | V6.2, V6.6b | A soft glow; punch words alone at twice the size; an emoji under the keyword; a hook title over the first 2 seconds, in words of mixed size. From V6.6b: four display fonts, six presets with our own names, and captions one word or one to three words at a time |
| Following the speaker | V6.5 | In a two-person shot, the frame cuts to whoever speaks, or shows both (§12.10) |
| Colour and light | V6.6a | Each shot's exposure, white balance and contrast corrected only as far as it needs; different cameras matched; a soft light on the face and a light vignette. A good picture is left as it is |
| Sound | V6.7 | Whooshes, pops, clicks and risers on every visual entry, made by Offcut itself, so no licence; the person's own music, ducked under speech; the voice tuned by measured need |
| A quality check | V6.8 | Before a short is shown, a check finds any stretch where nothing changes for 2.5 seconds, a visual with no sound, or the wrong person on screen, and fixes what rules can, as ordinary items the person can switch off. It never blocks an export |

**The Enhance panel (the founder's request).** A creator with a good microphone or good lighting may not want cleanup at all. The review has an Enhance panel with one switch per enhancement: **Voice cleanup, Motion, Colour & light, Sound effects, Music.** Each enhancement measures first and says when it has little to do ("Your audio is already clean: light touch"; "Your picture already looks right"). A switch flips the preview at once and is one undo step. Voice cleanup off keeps only the levelling to the platforms' loudness. The panel remembers the last choice in this browser for the next video, and settings has "Back to recommended". With every switch off, the short looks and sounds as it did before these features existed.

**Not at launch (founder, 2026-11-09):** HDR input (rejected with advice, §9.2), a bundled music library (the person's own track only), colour looks and LUTs (§21 item 16), sliders of any kind.

**Measured by.** E-16 (a blind test against Submagic before the founding release), E-15 (the quality pass), and E-11 (which switches people turn off).

---

## 13. Growth Plan and Numbers (Explicit Assumptions, Arithmetic Checked)

### 13.1 Calendar anchor

**v1.3 moves the calendar.** V6, then V6.1-V6.11, then V7-V10 put public launch around February 2027 (ESTIMATE, `buildplan.md` §0); the founder re-sets the date when V6.11 closes. Below, read each date as moved by the same amount: Month 1 starts at launch, M3 is 13 weeks after it and M4 is 26 weeks after it. **As first written: Month 0 = build and stabilization**, weeks 1-10 (12 Oct to 20 Dec 2026, A-2), based on implementation (5 weeks) + stabilization (5 weeks) from §9.9. **Month 1 = the first 30 days after public launch** (from 21 Dec 2026). The growth model starts at Month 1; Month 0 contains E-1, E-2, E-5 so that demand evidence precedes launch.

### 13.2 Rate definitions (all ASSUMPTION)

| Rate | Conservative | Base | Stretch | One-line justification |
|---|---|---|---|---|
| Visitor → signup | 2% | 4% | 6% | Free tool with try-before-signup; signup is required only at export. Guess; E-1 and landing analytics replace it |
| Signup → paid | 2% | 4% | 6% | Typical freemium range is low single digits (*general-market knowledge, re-verify*); E-5 gives an early reading |
| Monthly churn (applies after the acquisition month) | 12% | 8% | 6% | Creator tools are prone to "I stopped posting" churn (§14); guess |
| Blended revenue per paid user | $13.80 | $14.25 | $14.55 | Creator only at $15 monthly / $12 annual-equivalent; annual share 40% / 25% / 15% → 15 × (1 − share × 0.20) = 13.80 / 14.25 / 14.55 |
| Total six-month visitors | 6,000 | 12,000 | 25,000 | Founder-led distribution with no existing paying audience; guess and the biggest uncertainty |

### 13.3 Arrival-weight assumption by month

| Month | 1 | 2 | 3 | 4 | 5 | 6 | Total |
|---|---|---|---|---|---|---|---|
| Share of six-month visitors (ASSUMPTION; ramps as content compounds) | 10% | 12% | 15% | 18% | 22% | 23% | **100%** |

Customers acquired in month m are active at the end of month m; churn applies from the next month. Active paid at the end of Month 6 = Σ (new paid in month m) × (1 − churn)^(6 − m). For each scenario this equals total gross paid × a survival factor: Conservative 0.7899, Base 0.8534, Stretch 0.8875 (computed from the weights above).

### 13.4 Outcomes at Month 6

Formulas: Signups = Visitors × visitor-to-signup; Gross paid = Signups × signup-to-paid; Active paid = Gross paid × survival factor; MRR = Active paid × blended revenue per paid user.

| Scenario | Visitors | Signups | Gross paid | Active paid after churn | MRR |
|---|---|---|---|---|---|
| Conservative | 6,000 | 120 | 2.4 | 1.9 (2.4 × 0.7899) | **about $26** (1.896 × 13.80) |
| Base | 12,000 | 480 | 19.2 | 16.4 (19.2 × 0.8534) | **about $234** (16.386 × 14.25) |
| Stretch | 25,000 | 1,500 | 90 | 79.9 (90 × 0.8875) | **about $1,162** (79.876 × 14.55) |

**Base case month by month (churn 8%)**

| Month | Visitors | Signups (4%) | New paid (4%) | Active paid (prev × 0.92 + new) | MRR (× $14.25) |
|---|---|---|---|---|---|
| 1 | 1,200 | 48 | 1.9 | 1.9 | $27 |
| 2 | 1,440 | 57.6 | 2.3 | 4.1 | $58 |
| 3 | 1,800 | 72 | 2.9 | 6.6 | $94 |
| 4 | 2,160 | 86.4 | 3.5 | 9.6 | $136 |
| 5 | 2,640 | 105.6 | 4.2 | 13.0 | $185 |
| 6 | 2,760 | 110.4 | 4.4 | 16.4 | $234 |

(Visitors check: 1,200 + 1,440 + 1,800 + 2,160 + 2,640 + 2,760 = 12,000.)

### 13.5 What the headline target requires

**$1,000 MRR at base rates.** Active paid needed = 1,000 ÷ 14.25 = 70.2. Gross paid = 70.2 ÷ 0.8534 = 82.2. Signups = 82.2 ÷ 0.04 = 2,055. Visitors = 2,055 ÷ 0.04 ≈ **51,400 over six months**, about 4.3× the base-case traffic of 12,000. At stretch conversion rates (6% / 6%, 6% churn, $14.55), the need is about 21,500 visitors. **So $1,000 MRR is a month-12+ goal, not a month-6 base case**, unless traffic or conversion exceeds base by a factor of about four.

### 13.6 Unit economics sanity check (base case)

| Item | Value | Basis |
|---|---|---|
| Monthly Creator net of fees | $15 − (5% × 15 + 0.50) = $13.75 | A-9 |
| Annual Creator net of fees per month | ($144 − (5% × 144 + 0.50)) ÷ 12 = $11.36 | A-9 |
| Blended fee per paid user per month | 0.75 × 1.25 + 0.25 × 0.642 = about $1.10 | A-9 |
| Blended net per paid user per month | 14.25 − 1.10 = **$13.15** | |
| Lifetime value | 13.15 ÷ 0.08 = **about $164** | margin ÷ churn |
| CAC (cash) | $0 | A-3; paid acquisition is not used; founder time is the cost |
| Break-even on fixed costs | Fixed costs are $0, so **the first paying user** | A-10 |

The business has no cash costs to cover: the only fee is the merchant's cut of each sale. It does not cover a founder's living costs at the base case.

### 13.7 Month-by-month plan (distribution-weighted)

Because building is fast, after launch the plan puts about 60% of weekly time into distribution and user conversations and 40% into bug fixes and quality.

| Month | Goals | Channels | Measured | Go/no-go gate |
|---|---|---|---|---|
| **Month 0** (weeks 1-10) | Week 1-3: E-1 smoke test, spike E-3/E-4. Weeks 4-10: build; run E-2 concierge (weeks 6-9) and E-5 presale (weeks 8-10) | Build-in-public posts with before/after clips (hand-made for E-1); direct outreach to 50 technical creators | Waitlist signups, concierge outcomes, presale count, spike performance | **M0** (end week 3): E-1, E-3, E-4 pass or stop. **M2** (end week 10): launch gate §20.1 |
| **Month 1** | Launch; first 100 signups; fix launch bugs; 15+ user conversations | Show HN / Indie Hackers / Product Hunt (one launch, not three), X and LinkedIn posts made with the product, DMs to concierge participants | Visitors, capability-pass rate, activation, reject reasons, errors | If activation < 40% of valid uploads (TARGET), pause marketing, fix the funnel |
| **Month 2** | First 4-8 paying users; raise activation and repeat | Weekly dogfooded clips; reply-with-a-video offer to commenters; creator communities | Activation, 14-day repeat, edit counts per clip, event keep rate (E-7) | If visitors < 50% of plan (< 600 for Month 2), shift effort to outreach |
| **Month 3** | M3 decision | Same + first case-study clip with a real user (with permission) | All §20.5 metrics | **M3 gate** (§20.5) |
| **Month 4** | Improve retention; decide P1 list | Content on "how it works"; comparison pages | Churn reasons, music requests, browser blocks (E-8) | Promotion triggers (§9.5) |
| **Month 5** | Ship the first justified P1 item | Referral nudge (one prompt after the third export) | Retention by cohort | None beyond Month 3/6 gates |
| **Month 6** | M4 decision | Annual-plan push to improve retention | MRR, churn, activation | **M4 gate** (§20.5) |

### 13.8 If this undershoots

Most likely cause: **traffic**, not conversion. The base case needs only 12,000 visitors but zero audience is assumed (A-5).

| Symptom | Response |
|---|---|
| Visitors < 50% of plan at Month 2 | Switch from broadcast to direct outreach and partnerships (dev-rel influencers, newsletters), and publish more product-made clips |
| Visitors on plan, signup rate < 2% | Run the E-6 messaging A/B again with a new hero; test showing results before sign-up |
| Signups fine, paid < 2% | Raise value (more event types), lower friction at J9, or test $12 (price is a hypothesis, H2) |
| Activation low | Treat as a reach/performance problem first (E-8, E-3): browser blocks and timeouts, not marketing |
| Retention low | Look at why users stop posting vs. dissatisfaction with output (§14) |

### 13.9 The 3,000-paid-user goal (founder, 2026-11-01)

The founder's goal is 3,000 paying users, as fast as possible. The arithmetic, at 8% monthly churn and $14.25 blended revenue:

| Reach 3,000 in | New paid users a month | Visitors a month at 0.16% (4% × 4%) | Visitors a month at 0.48% (8% × 6%) |
|---|---|---|---|
| 12 months | 380 (3,000 ÷ 7.90) | about 237,000 | about 79,000 |
| 24 months | 277 (3,000 ÷ 10.81) | about 173,000 | about 58,000 |
| To hold 3,000 at steady state | 240 | 150,000 | 50,000 |

MRR at 3,000 is about **$42,750**. **This is a multi-year goal for a solo founder with no audience unless a channel takes off.** The §13.4 base case still stands as the early reading. Nothing here is a forecast. The levers that move it, biggest first: distribution through creators (affiliate at 30% recurring, ASSUMPTION), free-tier shorts that carry the watermark into public feeds, a better visitor-to-paid rate from a clearer offer (E-14), and churn.

**Channels for the long path, in order to test.**

| Channel | Why it may work | Measure |
|---|---|---|
| Five free shorts from a creator's latest video, in exchange for a public before-and-after | Proof on their real audience | Replies, posts, signups per creator |
| "Mine your back catalogue": the offer credits cannot match | The clearest difference | Creator-plan conversion |
| Affiliate deal for mid-size creators | Pays only on results; A-3 allows it | Paid users per affiliate |
| The watermark on free exports | Each one is a public demo | `landing_view` from referrals |
| Comparison pages (OpusClip, Submagic alternatives) | Searchers already want this | Search visits, conversion |
| Show HN: the engine in a browser, nothing uploaded | Technical virality | Visits, waitlist |
| The founder's own Shorts about making Shorts | No budget needed | Views, visits |

**Marketing does not wait for V6.11.** Start after V6.4 with alpha users, and after V6.8 with public demos (v1.5; v1.3: V6.6) (`buildplan.md` §8A.0).

---

## 14. Retention & Churn

**Why users return.** Usage cadence follows posting cadence: the primary ICP posts 2-5 times per week (ASSUMPTION), so the product is a weekly habit if it saves time every time. Each clip is a new job, and nothing needs to be re-learned thanks to default styles.

| Likely churn cause | One mitigation |
|---|---|
| Creator stops posting (the dominant cause, ASSUMPTION) | Pause subscription for up to 3 months instead of cancel (P1); annual plan offer at cancellation |
| Output quality plateaus; every video looks the same | Add event types according to E-7 evidence; allow style switching (already at MVP) |
| ASR errors on accents or noisy audio frustrate | E-10 accuracy gate before launch; word-edit UI; show the edit count as a quality metric |
| CapCut or another free tool matches the feature | Keep the speech-driven visuals clearly better; track event keep rate; do not compete on price |
| Browser/device failures (timeouts, crashes) | Error taxonomy with a "report this" button that sends only the allowlisted diagnostic; fix top 3 errors weekly |
| Low usage versus price (posting once a month) | Offer a monthly-cancel flow with a Free downgrade, not friction |
| YouTubers finish their back catalogue and need the product only weekly (v1.3) | Weekly value must stand alone: creator memory improves the fit; the annual plan is offered once the queue finishes |

---

## 15. Technical Architecture (summary only)

The tech spec (`technicalspec.md`) is authoritative. Every high-complexity element below has a row in §9.7 and a justification in §9.8.

```text
                      INTERNET (thin)
        ┌───────────────────────────────────────┐
        │  Vercel: app shell, /api/v1 proxy     │
        │  Asset storage (free): model files    │
        │  Render API: auth, billing webhooks,  │
        │       entitlements, usage receipts,   │
        │       allowlisted analytics,          │
        │       /plan, /shorts, /chat-edit →    │
        │       AI provider (v1.2, v1.3)        │
        └───────────────────┬───────────────────┘
        no video, audio, frames or project data; text + timings only in the AI request bodies
 ══════════════════════════ ╪ ═══════════════════════════
                       USER'S BROWSER
   React/TS UI ─ Workers ─ Rust/WASM core
     │             ├─ Decode / probe
     │             ├─ ASR (local model)
     │             ├─ Audio chain → program audio (cuts, music, ducking)
     │             ├─ Edit plan: validate (AI plan) or produce (rules)
     │             ├─ Scene graph (video as a node) → Vello renderer
     │             └─ WebCodecs encode + MP4 mux
     └─ OPFS / IndexedDB (clips, transcripts, plans, edits, music, brand kit, models)
```

- **Client:** React/TypeScript UI; Rust compiled to WASM; Web Workers; WebGPU (Vello); WebCodecs; OPFS/IndexedDB.
- **Hosting (all free tiers, A-10):** the React + Vite frontend is static files on Vercel, which also proxies `/api/v1/*` to the backend so the API is same-origin. The backend is one stateless Rust/Axum service on a Render free web service. Postgres is a free managed instance from a separate provider (not SQLite: Render's free tier has no persistent disk, and Render's own free Postgres expires). Model files, the demo video and the sample clip sit on free-egress object storage. A free transactional email provider sends magic links, and the merchant of record sends webhooks.
- **Costs of free tiers, accepted:** the API sleeps when idle and takes about a minute to wake (a free uptime monitor keeps it warm, and import, processing, preview and review never need the API); there are no database backups; Vercel's free plan is restricted to non-commercial use, which is an open risk once checkout is live, and the only remedy is another free static host (§18).
- **Server-side data stored:** user email, plan and subscription status, usage receipts (export id and timestamp), processed webhook IDs, session records, magic-link token hashes (kept 1 day), waitlist and "tell me when" emails, and allowlisted anonymous analytics (§16).
- **NEVER stored or sent:** video, audio, music files, brand logos and fonts, captions as edited, project edits, file names, filesystem paths, or any user-generated text (§17 enforces by test). **Sent but never stored (v1.2):** the transcript text, word timings and prosody scores, in the body of the plan request only, forwarded to the AI provider and dropped when the response returns; never logged.
- **Scale ceiling and later steps:** a single small server plus static hosting handles thousands of users because there is no media compute. Any scaling step beyond the free tiers (a larger instance, a read replica, separate analytics ingestion) costs money and is outside the MVP; it needs a founder decision. No distributed system is justified at MVP.

---

## 16. Data Model (conceptual)

**MVP, server**

| Entity | Key fields | Notes |
|---|---|---|
| User | id, email, created_at | Email stored normalized (trimmed, lower-case; V6: the sign-in link keeps only that form); deletion flow in §17 |
| Session | id, user_id, refresh_token_hash, expires_at | Refresh tokens stored only as hashes; rotated on use |
| Subscription | user_id, provider_customer_id, plan, status, current_period_end | Source of truth is the merchant of record; webhooks update this |
| WebhookEvent | provider_event_id (unique), type, processed_at | Idempotency: unique key prevents double-processing |
| UsageReceipt | export_id (unique, idempotency key), user_id, created_at, plan_at_export | No content, no file info |
| AnalyticsEvent | anon_id, name (allowlisted), props (allowlisted), ts | 90-day retention (ASSUMPTION); never joined to a user at MVP (there is no user_id column and no consent UI) |
| MagicLink | token_hash, email, expires_at, consumed_at | Only the hash of the link token is stored; rows purged after 1 day |
| AiUsage (v1.3, replaces PlanUsage) | user_id, month, kind (plan / shorts / chat), count | Requests per signed-in user per UTC month; no content. Anonymous requests are limited in memory per network, never stored |
| AiShortsGrant (v1.3) | user_id, month, hash of the file's prefix hash | So a failed run of the same video is not charged twice; no content |
| AiSpend (v1.3) | month, amount | Total provider cost, to enforce the cap; no user link |
| PlatformWaitlist | email, wanted (launch / safari / firefox / mobile / linux) | E-1 waitlist and the unsupported-page email field (§9.3) |

**MVP, client storage**

| Entity | Key fields | Notes |
|---|---|---|
| ClipRef | id, OPFS handle, probe metadata (duration, resolution, codecs) | Source stays in OPFS; deleted with "Clear local data" |
| Transcript | words[], sentences[], model_version | Never stored on a server; sent only in the plan request while "AI edit plan" is on (v1.2, §12.8) |
| DetectedEvents | list with params, confidence, enabled | Re-computed on caption edit; the rules producer's input to the plan |
| EditPlan (v1.2) | source (AI or rules), catalog version, cuts, items, chapters, text extras, music cue | Stored per clip; never re-requested except by "Regenerate plan" |
| EditState | word edits, item toggles, cut restores, filler-cut switch, style, crop offset, music choice, brand kit choice | User choices only |
| MusicTrack (v1.2) | id, OPFS ref, duration, loudness | A user-supplied file (v1.5: no bundled tracks at launch); stays local |
| BrandKit (v1.2) | logo ref, colours, font refs | One per account, stored in this browser only (no sync at launch) |
| Source (v1.3) | id, OPFS ref, probe metadata, stage, windows done | A long video; no file name |
| MasterTranscript, ShotSummary (v1.3) | words, sentences, shots | Never stored on a server; sentence text is sent only in the Director request |
| Short (v1.3) | id, source id, title, style, reason, parts, state | Each opens as a clip |
| Asset (v1.3) | id, OPFS ref, kind, label, note | The user's B-roll; label and note go in AI requests |
| CreatorMemory (v1.3) | counts by kind and style, length buckets | Numbers only; off switch and reset |
| RenderCache | key (hash of inputs + renderer version), output ref | Evictable |
| ModelCache | name, version, sha256, size | Verified on load |
| Entitlement | signed token, expires_at, free_exports_remaining | Cached up to 7 days |

**P1:** VersionSet, CloudJobConsent, brand-kit and memory sync across browsers. **P2:** Workspace/Team, local LLM model cache.

---

## 17. Security & Privacy

| Area | Commitment |
|---|---|
| **Auth** | Email magic link (no passwords at MVP) with short-lived link tokens; optional Google sign-in is P1. The email address passes through the mail provider, Brevo, to deliver the link (V6) |
| **Session handling** | Short-lived access token, rotating refresh token in an HttpOnly, Secure, SameSite cookie; refresh tokens stored only as hashes server-side |
| **Rate limiting** | Per-IP and per-account limits on sign-in, magic-link sends, usage receipts and webhooks (token bucket) |
| **Web security** | Strict Content Security Policy, no third-party scripts in the app shell, COOP/COEP for WASM threads, model files verified by SHA-256 before load |
| **Privacy documentation** | Plain-language Privacy Policy and the "What leaves your device" page (§12.7), written to match the allowlist exactly; both reviewed whenever the allowlist changes |
| **Analytics rules** | Allowed: anonymous event names and numeric/enum properties from an explicit list (capability pass/fail, reject_reason, stage timings, error codes, export success, plan outcome and item/cut counts). **Never allowed:** file names, transcript text, caption text, event or plan params, audio/video content, free-text user input. Enforced by an automated browser test that processes a fixture clip while capturing all network traffic: it fails if any request goes to a non-allowlisted host, if any payload other than the one plan request contains a transcript word, if any payload contains media bytes, or if a request is sent during processing other than analytics beacons, the API warm-up and the plan request |
| **Edit-plan request (v1.2)** | Only from the plan use-case; body = transcript text, timings, prosody scores, catalog version; no media, no file name, no user edits. The server holds it in memory for the request only, never writes it to the database or logs, and forwards it only to the configured AI provider over HTTPS. The provider's retention and training terms are recorded by TE-15 and shown on the settings page; a provider that trains on the data or keeps it is not chosen unless the founder accepts that in writing and the settings page says so |
| **AI requests of the long path and chat (v1.3)** | One use-case per route; bodies carry sentence text and timings, shot categories, the typed message and short history, the words of the short, asset labels and memory counts. **Never a frame, an audio sample, a file name.** Held in memory for the request only, never written to a database or log. Chat text is never in analytics |
| **A long video's sound (v1.4)** | Always transcribed in the cloud, in compressed pieces of at most 90 seconds, through `POST /transcribe` under the same rules as a clip's (§12.6): memory only, never logged, the provider's retention off. The long page says so before the work |
| **Stock search (v1.4, V6.8)** | Only while "Stock B-roll (Pixabay)" is on; the words go to our server and Pixabay only, are never stored or logged by us, and results are cached for a day under a hash of the words, as Pixabay's terms require. The chosen clip is downloaded from Pixabay's server straight into the browser |
| **Legal-review gate** | Before any public claim of GDPR, India DPDP Act, CCPA, SOC 2 or similar compliance, or any absolute wording such as "never leaves your device", a lawyer reviews the claim (*general-market knowledge, re-verify before launch*). The MVP's factual claim is "your video is not uploaded to our servers," backed by the network test; from V6.2 the copy also says that the transcript is sent to make the edit plan, with the provider named. Cloud transcription (§12.6, built in the cloud transcription release) needs its own review before launch; until then the founder approved its wording (2026-11-06). v1.4 adds the long path's always-cloud sound and the stock search to that review |
| **Deletion/export** | "Delete account" first cancels any active subscription, then immediately removes the user, sessions, subscription link, usage receipts and waitlist entries from our servers (well inside a 30-day bound); analytics hold no account link, so there is nothing to remove there and they expire at 90 days; billing records may be retained by the merchant of record for tax reasons and are out of our control. "Clear local data" removes OPFS/IndexedDB content in that browser but cannot remove files the user already downloaded. "Export my account data" returns a JSON of server-side records only (there is nothing else) |

---

## 18. Key Risks

| Risk | Impact | Early signal (observable) | Mitigation (concrete action) |
|---|---|---|---|
| **Nobody wants it** (built before demand is proven) | Months of work, no revenue | E-1 waitlist < 5% of ≥ 1,000 visitors; E-2 < 8 of 15 would post the output; E-5 < 5 presales | Run E-1, E-2, E-5 in Month 0; the M0 gate (end of week 3) and the M3/M4 decision rules (§20.5) are pre-committed |
| **Complexity debt** (a large LLM-generated codebase the founder cannot confidently debug) | Slow fixes, regressions, abandoned codebase | Bug fix time > 2 days for ordinary issues; duplicated logic; tests flaky | Keep the module boundaries in the tech spec; require tests with every module (golden frames, detector precision tests); weekly deletion pass for dead code; the principle "readable over clever"; a "can I explain this module?" review per milestone |
| **Stabilization and operational burden exceeding the estimate** | Launch slips; support eats growth time | Stabilization week 5 still has > 10 open P0 bugs; corpus pass rate < 90% at week 8 | Fixed 5-week stabilization with the corpus pass-rate gate; scope-cut the P0 features with the lowest H1 contribution before slipping the date; support hours capped (≤ 5/week) with an auto-reply FAQ |
| Local performance misses budget on user hardware (H3) | Users abandon during processing | Median pipeline time > 180 s on R1 (E-3, E-4); abandonment during J6/J10 | Pre-agreed fallbacks (§20.4): smaller model, Canvas2D overlay path, lower preview resolution; telemetry of stage timings |
| ASR errors on accents/noise create ugly captions | Edits eat the time savings; H1 fails | E-10 WER > 12%; median caption edits > 8 per 60 s | Test on accent corpus before launch; caption edit UI; model upgrade path in model manager |
| Visual-event false positives (a wrong number on screen) | Trust loss on first use | Detector precision < 90% on hand-labeled transcripts; users turn off > 40% of events | Precision-first thresholds; per-event toggle; labeled-corpus regression tests; log only enum outcomes (toggled/kept), never content |
| **Platform reach** (desktop Chromium only) | Many creators cannot use it | E-8: > 40% of visitors fail the capability check or are on mobile | Honest unsupported page with email capture; E-8 decides P1 promotion of Safari/mobile |
| WebGPU/Vello/WebCodecs variance or readback slowness | Rendering fails or is slow on some GPUs | Export failures concentrated on specific GPU/driver combinations; E-4 miss | Spike in week 1-2; Canvas2D overlay fallback; allowlisted GPU/vendor enum logged (no fingerprinting beyond enum) |
| **Acquisition channel**: build-in-public reach too small | Traffic far below the base case | Month 1 visitors < 600 | Direct outreach plan; partner posts; use the product to make the demo clips; E-1 gives an early reach reading |
| **Competitor response** (CapCut/Submagic add number visuals or on-device mode) | Differentiation erodes | A competitor release adds numeric animations; our event keep rate drops in comparisons | Compete on precision, privacy and flat pricing; move faster on event types; do not rely on one differentiator |
| Payments friction (merchant-of-record onboarding, payouts, chargebacks) | Cannot collect or has tax headaches | Account approval delay > 2 weeks; refund rate > 10% | Start onboarding in week 1; fallback to a second provider; 14-day refund policy |
| Privacy-claim regression (a bug sends transcript text anywhere but the plan request, or media anywhere) | Legal and reputational damage | The network test fails in CI | Block releases on the network test; analytics allowlist code-reviewed |
| **AI plan quality** (v1.2): a wrong emphasis, an odd card, a cut that eats a beat | Output feels robotic; trust drops | E-11 keep rate under 60%; cuts restored on more than 20% of clips | Validator drops impossible items; numbers come only from the transcript; every item and cut is a toggle; prompt and catalog tuned against the labeled transcripts |
| **AI provider dependence** (v1.2): free tier shrinks, terms change, outage, latency | Plans fall back to rules for everyone; quality drops to v1.1 | Fallback rate above 10%; p90 plan latency above 20 s; a terms notice | Provider behind one adapter (`technicalspec.md` §40.7); a second candidate tested in TE-15; rules fallback always works; a paid tier is a founder decision |
| **Breaking what works (v1.3):** new paths regress the proven clip path | Trust, and months of proven work lost | A protected test changes without a declared row | The compatibility contract (`technicalspec.md` §41): flags, identity constructors, both-path test runs, stop-and-ask |
| **The Director picks badly (v1.3):** shorts that do not stand alone, or joins that change meaning | H5 fails: the headline feature is a pile of fragments | E-13 under target; parts deleted on over half of shorts | A labelled set of real videos; a stronger model; fewer joins; whole-sentence rule always holds; the founder re-plans |
| **Long jobs die on real computers (v1.3):** sleep, tab discard, memory, disk | Users lose an hour and leave | TE-18, TE-20; `source_stage_timing` failures | Checkpoints, wake lock, disk check, honest copy; at worst, cap at 10 minutes |
| **Competing on OpusClip's home ground (v1.3)** | Compared on speed and convenience, where local loses | Lost deals citing speed; low long-path activation | Lead with stitched shorts, whole sentences, instant edits and the back catalogue; keep the clip path (§7) |
| **Scope (v1.3):** eleven releases before launch, with no customer yet | Months built on an untested guess | E-12 not run; no alpha by V6.4 | Alpha after V6.4; public demos after V6.8 (v1.5); the founder may reorder or drop any later release |
| **AI cost above plan (v1.3)** | The cap trips and plans fall back to rules for everyone | `ai_spend` at 80% early in a month | Cheaper model for per-short plans; lower Free quotas; the founder raises the cap |
| **Model and code licences (v1.3)** | A GPL model or copied code forces a rewrite or a takedown | TE-23, TE-24 readings | Licence check is a pass line of each; study Tooscut for approach only, never copying (Elastic License 2.0). v1.4: Tooscut is the main study source for the editing engine, under the same rule |
| **Outside services on the long path (v1.4)** | A long video cannot be transcribed when Groq is down or over budget (no device fallback, by choice); stock B-roll stops if Pixabay changes its terms | `source_paused` events with `busy` or `unavailable`; TE-26 | Pause with Continue, never fail; Groq's paid tier and configurable budgets; stock behind its own switch and flag, with own clips unaffected |
| Odd real-world inputs (VFR, rotation metadata, odd audio) | Broken exports for a slice of users | Reject or error rate > 15% of attempts | The 40+ clip corpus, specific rejection messages, fix top error weekly |
| **Free-tier dependence** (A-10: nothing is paid for) | A provider changes or enforces its terms and a component stops working; Vercel's free plan is restricted to non-commercial use; without a purchased domain, magic-link emails may land in spam | A terms notice from a host; API cold starts over 70 s; magic-link delivery under 95% | Each component can move to another free provider without code changes; the uptime monitor keeps the API warm; free-tier facts are re-verified before launch (tech spec TE-5 to TE-11). No paid fallback exists, so a failure here can delay launch |
| Launch timing near the holidays | Lower attention, slower feedback | Month 1 traffic below plan | Keep the launch date but time the Show HN/Product Hunt launch for the first week of January if Month 0 ends clean (ASSUMPTION; slightly delays the model) |

---

## 19. Open Questions

### Critical

| Question | Experiment | Milestone | Measurable question | If yes / if no |
|---|---|---|---|---|
| Do technical creators want auto-visualized numbers/lists enough to engage? | **E-1** Landing page + 3 hand-made demo clips + waitlist, 3 weeks | M0 | ≥ 1,000 visitors and ≥ 5% join the waitlist (≥ 50) (TARGET). Fewer than 1,000 visitors = inconclusive | Yes: continue. No: rewrite the pitch and re-run once; if still < 3%, stop and reconsider the ICP |
| Can local ASR + render fit the time budget on R1? | **E-3** (ASR perf) and **E-4** (render/encode perf), spike in weeks 1-2 | M0 | Median ASR ≤ 20 s and render+encode ≤ 90 s for a 60 s clip on R1 (TARGET); p90 total ≤ 180 s | Yes: continue. No: apply fallbacks (§20.4); if still failing, cloud fallback becomes P0 and H4 weakens |
| Will strangers pay before the product is polished? | **E-5** Founding-member presale at $99/year (31% off $144: (144 − 99) ÷ 144 = 31.25%). v1.5: starts right after V6.8, once E-16's blind test is read (founder, 2026-11-09; v1.3: right after V6.6, founder, 2026-11-02; `buildplan.md` §8A.6) | M2 | ≥ 5 paying strangers (not friends/family) before launch (TARGET) | Yes: confident launch. No: investigate price vs value before spending on marketing |
| Is the output good enough that people would post it? | **E-2** Concierge/alpha with 15 technical creators on their own clips | M1-M2 | ≥ 8 of 15 would post the result with ≤ 2 minutes of changes (TARGET) | Yes: H1 supported. No: identify the gap (captions? events? audio?) before launch |

### Important

| Question | Experiment | Milestone | Measurable question | If yes / if no |
|---|---|---|---|---|
| How accurate is ASR on accents and noisy audio? | **E-10** 20 clips (10 US, 10 Indian/other English accents), measure WER and caption edits | M1 | WER ≤ 12% (TARGET) and median edits ≤ 8 per 60 s | Yes: ship. No: larger model, or stronger edit UX, or limit claims |
| Are detected events kept? | **E-7** Event keep rate in alpha/beta (≥ 100 clips, ≥ 20 users) | M2-M3 | ≥ 60% of events kept; precision ≥ 90% on labeled transcripts (TARGET) | Yes: add event types. No: tune rules or reconsider the wedge |
| How many visitors can use the product at all? | **E-8** Capability-check beacon on landing pages (anonymous enum) | M0-M3 | ≥ 55% of visitors pass the capability check (TARGET) | Yes: stay desktop-Chromium-first. 40-55%: add Safari/Firefox earlier. < 40%: re-plan around mobile or desktop app |
| Is privacy a purchase driver? | **E-6** A/B on landing hero: outcome-led vs privacy-led, plus a one-question interview ranking | M3 | Directional difference in signup rate (low statistical power at this traffic: treat as qualitative) | Privacy wins: lead with it. Not: keep as supporting claim |
| Do users come back? | **E-9** 14-day repeat export among activated users | M3 | ≥ 35% (TARGET) | Yes: continue. No: study why before adding features |
| Is the AI edit plan better than the rules? (v1.2) | **E-11** Plan items kept vs switched off, cuts restored, and "Would you post this?" split by plan source (AI vs rules), in alpha and after launch | V6.2 → M3 | ≥ 60% of AI plan items kept; cuts restored on ≤ 20% of clips; "post as is or small edits" higher for AI plans than rule plans (TARGET) | Yes: keep the AI plan as default. No: tune the prompt and catalog; if still lower, make rules the default and the AI plan opt-in |

### Long path (v1.3)

| Question | Experiment | Milestone | Measurable question | If yes / if no |
|---|---|---|---|---|
| Is the stitched-shorts idea wanted, before it is built? | **E-12** (optional, founder's decision of 2026-11-02; recommended). One week beside V6: run one real video through OpusClip, Submagic, Vizard and Descript by hand to see whether any stitch moments; a throwaway script (transcript → LLM → ffmpeg) makes single-run and stitched shorts for 20 YouTubers free; offer the $99 founding plan | Beside V6, or any time before V6.4 | ≥ 8 of 20 post at least one; stitched shorts perform at least as well as single-run ones; ≥ 5 strangers prepay (TARGET) | Yes: confidence in V6.3-V6.5. No: lead with whole-sentence quality and the back catalogue, and re-plan V6.8-V6.11. **The build does not wait for this; the founder carries that risk knowingly** |
| Are the Director's shorts good? | **E-13** Panel rating on a labelled set of ≥ 10 real long videos, then real users: `short_action`, deleted parts, "Would you post this?" | V6.4 (set), M2.5, M3 | ≥ 60% of proposed shorts kept; parts deleted on ≤ 25%; ≥ 50% "post as is or small edits" (TARGET) | Yes: keep. No: tune the prompt, try a stronger model; if still low, offer fewer shorts; the founder decides |
| Which path pays? | **E-14** Conversion and retention split by first path used, plus landing message | M2.5 → M3 | Directional: paid and repeat rates by path | Marketing follows the winner. The other path stays unless it costs more than it brings |
| Do our shorts reach the benchmark? (v1.4) | **E-15** The founder's quality pass: 3-5 source videos made into shorts in Offcut and compared side by side with Submagic-made shorts and the caption references; the differences become a checklist | After V6.11, before V7 (`buildplan.md` §8A.12) | Every difference fixed, tuned, or knowingly accepted by the founder (TARGET) | Yes: the feature freeze holds and V7 starts. No: fixes are presets, timing and catalog tuning; a new kind is a stop-and-ask (`technicalspec.md` §41.4) |
| Do people prefer our shorts to Submagic's? (v1.5) | **E-16** A blind test: 3 source videos (one solo talking head, one two-person podcast, one raw clip) made into shorts by Submagic on its default style and by Offcut on its defaults, with no hand edits; 20 people who watch Shorts, each shown pairs in random order, unlabelled, choose the better short | After V6.8, before the founding release; repeated in the quality pass | Offcut chosen in at least 50% of pairs (TARGET; at least 60% makes "chosen by most viewers" a claim the landing page may use, with the method stated) | Yes: the founding release goes ahead. No: the reasons people give become E-15's first rows, and the founder decides whether the founding release waits |

### Minor

- Best annual discount (20% assumed): test after Month 3.
- Watermark placement and size: test with users in alpha.
- Exact model size/latency tradeoff: decided by E-3/E-10.
- Whether to add Google sign-in: after Month 3 if sign-in friction shows in analytics.
- Final product name: decide before launch (A-11).

---

## 20. Definition of Done (MVP)

### 20.1 Functional checklist (M2, launch gate)

- [ ] A fixture 60-second English clip goes from drop to downloaded MP4 on R1 and R2 without errors.
- [ ] The capability check passes on Chrome and Edge (Windows and macOS) and shows a specific unsupported page elsewhere.
- [ ] Every constraint in §9.4 has a test that produces its specific rejection message, except one speaker and English, which are stated at the drop zone and not detected at import (§9.4).
- [ ] Model download is resumable and hash-verified; a second session on the same browser skips it.
- [ ] Detector: ≥ 90% precision on a hand-labeled set of at least 30 transcripts for the four event types (TARGET); events below threshold are not created.
- [ ] Caption edits re-run detection for the affected sentence.
- [ ] Voice chain output measures at the target loudness within ±1 LU (target -14 LUFS, set in the tech spec and tunable there).
- [ ] With no cuts (plan off, or every cut restored), the exported MP4 has the same duration as the source clip (within one video frame) and the enhanced audio has the same duration as the source audio; a fixture with 0-5 s speech, 5-10 s silence and 10-20 s speech exports as 20 seconds with the pause intact and audio and video in sync on both sides of it.
- [ ] With cuts (v1.2), the same fixture under a fixed test plan exports at exactly the duration the time map states (within one video frame), no cut starts or ends inside a word, no click is audible at a splice (measured), and audio and video stay in sync on both sides of every cut.
- [ ] Edit plan (v1.2): a plan that names a missing word, an unknown item, an over-limit text or a digit not in the transcript has that item dropped and the rest kept; a number on screen always equals the parsed transcript value; with the API unreachable, the clip is processed with the rules plan and exports.
- [ ] Music (v1.2): with a music track, speech segments sit at least 12 dB above the music bed (TARGET, E-2 tunes it) and the mix measures at the loudness target within ±1 LU.
- [ ] Brand kit (v1.2): a kit's colours, fonts and logo appear in every caption preset, card and furniture item (golden frames per template).
- [ ] Preview plays audio and video in sync (drift ≤ 80 ms over 60 s, TARGET).
- [ ] Export produces 1080×1920 H.264/AAC MP4 for Creator and 720×1280 with watermark for Free; plays in the common phone gallery apps and uploads to at least two target platforms in manual tests.
- [ ] Free account: 3 exports per month enforced by the entitlement token; counter visible before each export.
- [ ] Checkout works end to end in test and live mode; webhook handling is idempotent (replayed events do not change state).
- [ ] **Privacy/network verification:** the automated test captures all network traffic while processing and exporting a fixture clip and passes: no media bytes anywhere; transcript text only in the body of the one plan request; with "AI edit plan" off, no transcript or caption text at all; only allowlisted hosts and events.
- [ ] "What leaves your device" page matches the allowlist and names the AI provider and its data-use terms exactly.
- [ ] "Clear local data" removes clips, transcripts and render cache.
- [ ] Corpus of ≥ 40 real clips (including VFR, rotated, noisy, accented) exports successfully at ≥ 90% (the rest rejected with a specific message, none crash).
- [ ] Error taxonomy and stage timings are logged through the allowlist; an uptime monitor is on the API.

**Long path (v1.3).**

- [ ] A 45-minute 1080p fixture (v1.4; v1.3: 20 minutes) goes from drop on the long page to at least 5 exported shorts on R1 and R2 without errors; each passes the verifier; the shorts reopen after the original file is gone.
- [ ] A 45-minute 4K HEVC fixture does the same on D1, or is rejected with a specific message on a machine that cannot decode it.
- [ ] With cloud transcription unavailable, a long video pauses with its reason and Continue, and finishes after Continue with nothing sent twice.
- [ ] B-roll placed by hand on a word, from the person's own clip and from Pixabay, exports and passes the verifier; each stock clip shows "From Pixabay".
- [ ] No exported short starts or ends inside a sentence; every number on screen is parsed from its own words.
- [ ] A short stitched out of order is in sync at every join, with no audible click.
- [ ] Killing the tab mid-transcription loses at most the windows in flight; after a reload the job resumes without the file, and a step that reads the video asks for the same file.
- [ ] With AI unavailable, a long video still gives a usable list, marked "Picked without AI".
- [ ] A moving speaker stays in frame across the fixture, with no pan across a cut.
- [ ] The privacy test covers the long path, chat, assets and stock: no frame or file name in any request, sound only in `/transcribe` windows, search words only in the stock search, and no asset bytes anywhere.
- [ ] **With every new feature off, frames and samples equal V5's (`technicalspec.md` INV-26).**
- [ ] Each person-mask effect has passed TE-25, or is off.

### 20.2 Performance budgets (TARGETS validated by E-3, E-4)

**Reference conditions:** 60-second clip, 1080×1920 or 1920×1080, 30 fps, H.264, about 150 words of speech; R1 (2021-class Windows laptop, 8 GB RAM, integrated GPU, Chrome stable, plugged in) and R2 (Apple M1 8 GB); model cached.

| Stage | Budget on R1 (TARGET) |
|---|---|
| Probe + audio extraction | 3 s |
| Local transcription | 20 s |
| Detection + scene build | 2 s (detection 1 s, layout and scene 1 s) |
| Audio chain | 4 s |
| Render + encode | 90 s |
| Mux + finalize | 2 s |
| **Median total** | **121 s** (3 + 20 + 2 + 4 + 90 + 2) |
| **p90 budget (promise: under 3 minutes)** | **180 s** (about 49% headroom over the median) |
| First run, model download | ≤ 90 s: a 260 MB model at 25 Mbps (A-8) takes 260 × 8 ÷ 25 = 83 s, plus about 5 s initialization = 88 s, leaving a margin of 2 s |

**Status on 2026-10-09 (D-67 of `docs/v2/v2implementation.md`).** The transcription budget is not met by the speech model V2 uses, the small-size one: about 150 s on an R1-class laptop for the 74.7 s reference clip, which is about 120 s for 60 s. The founder accepts that for the quality of the transcript, up to 145 s for 60 s. The budgets above stay the targets. On these numbers the median total is 3 + 120 + 2 + 4 + 90 + 2 = 221 s, so the three-minute promise does not hold until the model or the promise changes.

### 20.2A Long-path budgets (v1.3; v1.4, TARGETS, TE-17, TE-18)

Reference (v1.4): 45-minute 1080p H.264 source, R1, read in place, transcribed in the cloud, 8 shorts prepared, 5 exported at 60 s each.

| Stage | Budget |
|---|---|
| Probe and sound extraction (no copy of the file) | about 4 min |
| Vision over the whole file | 7 min |
| Cloud transcription | 6 min |
| Director | about 1.5 min |
| Keeping the chosen parts | 1 min |
| Prepare 8 shorts | 8 min |
| **To the list of shorts** | **at most 30 min** |
| Export 5 shorts | 7.5 min |

**If transcription misses:** more pieces in flight, Groq's paid tier, or a larger overlap between pieces (TE-18). **If long jobs fail on typical laptops** (TE-20): the copy tells people to keep the tab visible, and the founder decides. *(v1.3's reference was 20 minutes, copied whole and transcribed on the device: at most 55 minutes to the list, with a 10-minute cap as the fallback.)*

**(V6.1, read on D1.)** Probe and sound for 45 minutes read in place: 22 s (4K HEVC) to 55 s (5.6 GB 1080p), under the 4 minutes. Cloud transcription of 40 minutes: about 33 s of windows on Groq when not limited; Groq's free tier paused it once as busy (about 20 requests a minute), so the Dev tier comes before the alpha. The later stages are V6.2-V6.5's to read.

### 20.3 Critical-path arithmetic for the end-to-end promise

"About three minutes" = 180 s p90. The stages above are sequential in the worst case: 3 + 20 + 2 + 4 + 90 + 2 = 121 s median, leaving 59 s of headroom. Transcription and the voice chain can run in parallel, which improves the median, but the promise does not assume it.

These budgets and the three-minute promise are for the 60-second reference clip. A 90-second clip (the maximum, §9.4) is expected to scale roughly linearly, about 1.5× (around 180 s median), and has no budget of its own; stage timings report it.

### 20.4 Pre-agreed fallbacks and decision points

| If this misses | Fallback | Decision time |
|---|---|---|
| ASR median > 145 s on R1 for a 60 s clip (the founder's decision of 2026-10-09, D-67 of `docs/v2/v2implementation.md`; the line was 40 s) | Ship the smaller model with stronger caption edit UX; keep cloud fallback as P1 trigger | M0 (end of week 3) and re-check at M2 |
| Render + encode > 150 s on R1 | Canvas2D overlay path instead of Vello readback; cap input at 60 s and 30 fps | M0 and M1 |
| Denoise exceeds the CPU budget | Loudness normalization + high-pass only | M1 |
| Corpus pass rate < 90% at week 8 | Cut the least-valuable P0 inputs from the supported list (e.g. 60 fps input) rather than slipping launch | Week 8 |
| E-1 < 3% waitlist conversion with ≥ 1,000 visitors | Rewrite the pitch once; if still low, stop at M0 | M0 |

### 20.5 Product validation criteria (a working MVP is not a success by itself)

All thresholds are TARGETS. Measure from public launch (planned as Mon 21 Dec 2026 when written; moved by v1.3, §13.1), excluding founder, friends and family.

| Question | Metric | Threshold |
|---|---|---|
| Users complete the core workflow | Activation: valid uploads that reach a successful export | ≥ 60% |
| Users understand the value | Post-export one-question prompt "Would you post this as is?" (yes / with small edits / no) | ≥ 50% yes or small edits |
| Users return/repeat | A second export within 14 days among activated users | ≥ 35% |
| Users tolerate the limits | Capability pass rate; reject rate on valid-looking uploads | ≥ 55% pass; ≤ 15% rejected |
| A signal justifies continued investment | Active paying customers, independent of friends/family | Month 3: ≥ 5; Month 6: ≥ 16 (base case 16.4, §13.4) |

**Decision rule**

| Date | Continue | Pivot | Kill |
|---|---|---|---|
| **M3: 21 Mar 2027** | Activation ≥ 60%, repeat ≥ 35%, and ≥ 5 active paid | Value signals met but paid < 2 (test price, ICP, channel), or paid ≥ 5 but repeat < 25% (fix retention/quality) | All of: paid < 2, repeat < 20%, activation < 40% |
| **M4: 19 Jun 2027** | ≥ 16 active paid and repeat ≥ 35% | 10-15 active paid or repeat 25-35%: change the ICP or price once and re-evaluate in 8 weeks | < 10 active paid (about $140 MRR) and repeat < 25% |

"Kill" means stop building features and stop spending on distribution. The engine can remain as a portfolio or open-source asset; no further product investment.

### 20.6 Later-phase checklist (public launch items)

- [ ] Terms of Service, Privacy Policy and refund policy live and reviewed (legal gate §17).
- [ ] Merchant-of-record account live, tax settings confirmed.
- [ ] Support inbox, FAQ and status page (the free uptime monitor's status page).
- [ ] Demo videos made with the product itself.
- [ ] Single coordinated launch (Show HN or Product Hunt, then social posts).
- [ ] SQL queries that compute the §20.5 metrics, run by hand against the database (no dashboard service is deployed).

---

## 21. Post-MVP Roadmap

| # | Item | Trigger | Precondition |
|---|---|---|---|
| 1 | Fix top stabilization issues; performance tuning | Error rate > 5% of attempts or p90 > 180 s | Stage timing telemetry live |
| 2 | Safari/Firefox support (P1) | E-8 shows > 25% of visitors blocked by browser | Feature detection stable; WebGPU/WebCodecs parity verified |
| 3 | ~~User-supplied music with speech-aware ducking~~ | Moved into launch scope (V6.7, v1.2) | |
| 4 | ~~Additional event types~~ | Moved into launch scope as catalog items (V6.6, v1.2); later additions follow E-11 | |
| 5 | Pause-subscription option (P1) | "Stopped posting" cited in > 30% of cancellations | Churn survey in place |
| 6 | ~~Face-tracking reframe~~ | Moved into launch scope (V6.5, v1.3) | |
| 7 | Pro plan ($29) with three versions per recording and brand presets (P1) | At least two Pro-worthy features exist and ≥ 20 paid users | Active paid ≥ 20; retention ≥ 35% |
| 8 | Mobile browser support (P1) | E-8 shows mobile is the largest blocked group and a viable path exists | Memory and encode budgets proven on mid-range phones |
| 9 | Cloud transcription | **Built** (cloud transcription release) | A lawyer's review of its wording before launch (§17) |
| 10 | Style DNA (P2); brand kit moved into launch scope (V6.10, v1.2) | Retention evidence that style consistency drives usage | Pro plan live |
| 11 | Local LLM producer of the edit plan (P2) | Provider cost or terms (TE-15), or privacy-led messaging wins E-6 | Device capability data shows adequate hardware share |
| 12 | Native desktop app (P2) | Web product validated and requests for offline/GPU speed | M4 continue decision |
| 13 | Multi-language (P2) | English retention validated; demand from a specific language | Accuracy testing per language; font and shaping support |
| 14 | Motion-graphic B-roll drawn by the renderer (diagrams, icons, charts) (P2). *v1.4: stock footage from Pixabay moved into launch scope (V6.8, §12.12), beside the user's own B-roll* | E-11 shows plan items are kept; users ask for drawn diagrams | A second video slot in the renderer (`technicalspec.md` §40.5) |
| 15 | Chapter markers, lower third (P2, founder's decision of 2026-11-02) | Users ask, or long-format output is added | `Chapter` already in the plan's format |
| 16 | Colour looks through 3D LUTs (P2) | Users ask for a consistent look | Renderer pass; closed preset list |
| 17 | Brand kit and creator memory sync across browsers (P1) | Users on two machines ask | A small server store; PS §17 review |
| 18 | Performance feedback from platform analytics (P2) | Retention evidence; users connect accounts willingly | OAuth (TS §36 excludes it until then). The long-term moat: plans learn what actually performed |
| 19 | Frames to the AI, as a labelled opt-in (P2) | E-13 shows clip choice suffers on footage-heavy videos | Consent UI; network test; privacy copy |
| 20 | Three or more people on screen; speakers found by voice (P2). *v1.5: two people on screen are followed from V6.5* | Demand after launch | Diarization; framing for more faces |
| 21 | Hindi and Hinglish captions (P2) | Demand from Indian creators; English retention validated | A non-English model; font and caption rules |
| 22 | HDR input (P2, v1.5: rejected at launch by the founder's decision of 2026-11-09) | Rejections of HDR are a top-3 reason in `reject_reason` | Our own tone mapping of 10-bit frames, read on real phone clips |
| 23 | A bundled music library (P2, v1.5) | Users ask; exports with music are rare | A bought pack with in-app rights, not registered with Content ID |
