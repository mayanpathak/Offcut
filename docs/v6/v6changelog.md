# v6changelog.md - record of every change made while building V6 and V6.1-V6.11

Entries are oldest first. Each states what changed, what differs from the plan, how it was checked, and what is open. V6's first prompt opens the build; Entry 0 below came before it.

## Open items

| # | Item | Owner | Where |
|---|---|---|---|
| 1 | `AI_MONTHLY_CAP_USD`, starting value 50 | Founder, at V6.2 | TS §52 |
| 2 | Provider and model, quality first | Founder, at TE-15 | TS §52 |
| 3 | Whether to run E-12 (optional, recommended) | Founder | PS §19 |
| 4 | Reorder or drop any of V6.8-V6.11 after the alpha | Founder | BP §8A.0 |
| 5 | Each person-mask effect after TE-25 | Founder | TS §51.4 |
| 6 | ~~A 10-minute cap, if TE-18 or TE-20 fails on real laptops~~ No longer applies: long sources are transcribed in the cloud (Entry 21, decision 6). TE-20 (hidden tab, sleep) still stands | Founder | PS §20.2A |
| 7 | The launch date, when V6.11 closes | Founder | PS A-2 |
| 8 | The long-path fixtures and labelled set of at least 10 real long videos | Founder, by V6.4 | BP §8A.4 |
| 10 | Dodo Payments' verification. **Read 2026-11-04:** identity and bank approved; the product review is on hold because the live landing page reads as a waitlist ("Offcut is not open yet", no way into the app), with a resubmission allowed once the site is public. Not V6's: live mode is the founding release's (D-285, D-303). The founder decides after V6 | Founder, after P150 | Entry 0.1, Entry 18 |
| 14 | Tell Dodo the pricing and legal pages are live, and check its product form's description against them | Founder | Entry 3 |
| 16 | Delete the webhook logger service on Render and its endpoint in Dodo's test mode (keep the `offcut-api` endpoint, registered 2026-11-04); remove `MAIL_API_KEY` from Render, as the V6 deploy is healthy | Founder | Entry 13, Entry 20 |
| 17 | V6 went live with P148's legal and pricing words and with the account page's two offers, which lead to Dodo's test mode while the pages say Creator is not on sale (D-285). Keep the offers or hide them until the founding release; decide with open item 10's resubmission | Founder, after V6 | Entry 17, Entry 20 |
| 18 | Spec revision v1.4 (TS §43, §45, §48; PS §12.9-§12.15; BP §8A) from Entry 21's decisions, before V6.1's plan. **Done 2026-11-08 (Entry 22);** the founder reads it | Agent, founder approves | Entry 21, Entry 22 |
| 23 | ~~Confirm or change the agent's 15 design choices of Entry 22~~ **Confirmed by the founder, 2026-11-08** | Founder | Entry 22, Entry 23 |
| 19 | Groq's Dev tier before the private alpha (45-minute sources are about 30 windows each) | Founder | Entry 21, D-342 |
| 20 | Re-read Pixabay's API terms and licence before V6.8 is planned | Agent | Entry 21 |
| 21 | ~~The Submagic benchmark shorts and the caption references, outside git~~ **Done:** six Submagic promos in `sh2clips/documents/submagic_clips/`, read 2026-11-09 (Entry 38) | Founder | Entry 21, Entry 38 |
| 22 | The local Tooscut copy (`sh2clips/old_project_study_editor/`) holds no licence file; keep upstream's licence text beside it so the study rule's basis is on disk | Founder | Entry 21, Entry 0 |
| 26 | D-428's three analytics events (`long_source_accepted`, `source_stage_timing`, `source_paused`) are not built: `analytics.rs`' tests pin the event list and no existing test may change tonight. Build them, editing those pinned lists, before `long_video` goes on in production (V6.4); nothing is lost while it is off | Founder decides, V6.4 | Entry 30, D-428 |
| 25 | HEVC clips on `/app`: the render worker (frozen in V6.1) configures its decoder with the `avcC` only, so V6.1 refuses HEVC on the clip page even with `high_res` (DC-2 holds for 4K H.264 only). Passing the `hvcC` through `frame-decoder.ts` reopens the render path: for the release that next opens it (V6.3, part copies) | Agent, V6.3 | Entry 29, D-427 |
| 24 | Groq's free tier answered 429 after 27 window requests in about 66 s (a per-minute limit, about 20), with the hour's audio far from spent. Two windows in flight (D-409) meet it on every 45-minute source. Before `long_video` goes on: the Dev tier (open item 19), or one window in flight | Founder, before V6.4's alpha | Entry 26, experiments.md TE-18 |
| 28 | ~~Confirm or change the agent's 15 design choices of Entry 38 (v1.5) before V6.2's plan is written~~ **Confirmed by the founder, 2026-11-11** | Founder | Entry 38, Entry 39 |
| 29 | E-16, the blind test against Submagic: 3 source videos, 20 viewers, after V6.8 and before the founding release | Founder, after V6.8 | Entry 38, PS §19 |
| 30 | At least 10 two-person podcast videos (wide two-shots) with the speaker marked per sentence, outside git, for TE-27 | Founder, by V6.5 | Entry 38, TS §46.6 |
| 31 | TE-29's side-by-side panel: 20 shots, graded and ungraded, unlabelled | Founder, at V6.6a | Entry 38, TS §47.4 |
| 27 | TE-18 (P181): at the 10 s overlap, 2 words lost and 3 changed within 2 s of the 29 seams of a 40-minute run, none doubled: the base rate of Groq's own variation (96 edits in 4,677 words), so the merge adds nothing measurable, but §13's "no word lost at a seam" fails by 2. Decide with V6.4's labelled long set before `long_video` goes on: accept, a larger overlap, or one window in flight | Founder, before V6.4's alpha | Entry 36, experiments.md TE-18 |
| 32 | ~~V6.2's founder questions F-1 to F-9~~ **Answered 2026-11-11:** yes to all; billing already on; 15 transcripts for TE-15; AI plans for signed-in people only (v6.2implementation D-493, D-494) | Founder | Entry 40 |
| 33 | **Closed 2026-11-11 (Entry 43): moot, DeepSeek replaces Gemini (v6.2implementation D-500).** Was: ~~The Gemini key's Google project is on the free tier~~ (P183: a 429 naming only `free_tier` quotas, Pro's at 0), although billing was thought on. Link the key's project to an active Cloud Billing account (or make a key in the billed project and put it in `.env` and `.env.deploy` as `PLAN_API_KEY`); AI Studio shows the tier. **No Gemini call until then (P183b).** Also recommended: turn off the developer logs in AI Studio's Logs and Datasets page (on by default for projects made on or after 2026-10-26, kept up to 55 days) | Founder, before P183b and P188 | Entry 42 |
| 34 | **Closed 2026-11-11 (Entry 43): moot, Gemini is not used (D-500).** Was: ~~Gemini 3.8 Flash's price doubles on 2027-01-01~~ (0.75 / 3.75 → 1.50 / 7.50 USD per 1M tokens): set `PLAN_PRICE_IN_USD_PER_MTOK` and `PLAN_PRICE_OUT_USD_PER_MTOK` on Render again that day, or the spend cap counts low | Founder, 2027-01-01 | Entry 42, v6.2implementation D-463 |
| 35 | **The privacy page and the settings table name DeepSeek before `ai_plan` goes on in production.** The privacy page promises "We will update this page before any new kind of data leaves your computer"; its "Cloud captions" style section, its "Who handles it" list and the settings "What leaves your device" table gain a row: the words of the transcript and their timings go to DeepSeek, in China, to plan the edit; never the audio or video. They cannot say "not kept" or "never used to train" (DeepSeek's terms state neither; D-492's note). The landing page does not change (the founder). Optional: write to DeepSeek to opt the account out of training, a right its privacy policy names | Founder (wording at CP 3) | Entry 43, v6.2implementation D-492, D-500 |
| 36 | **Closed 2026-11-12 (Entry 48): renamed with the founder's yes.** Was: **DeepSeek's key is `PLAN_API_KEY2`** in `.env` and `.env.deploy`; D-465's configuration reads `PLAN_API_KEY`. May the agent rename it, replacing the Gemini line (name only, value never printed)? | Founder, before P189 | Entry 43, v6.2implementation D-465 |
| 37 | **Closed 2026-11-12 (Entry 49): the founder chose `deepseek-flash`, thinking off, at 0.30 / 1.20 USD per 1M tokens; yes to the revision (done in P188b) and to item-level parsing (P190).** Was: **Checkpoint 1 (P188): choose the model, confirm the prices, and say yes or no to the prompt revision and item-level parsing.** Read `~/v6.2-review/plans/README.md`'s ten plans first, then `docs/v6.2/experiments.md` TE-15's full read. Recommended: `deepseek-flash`, thinking off; one prompt revision and Flash re-read (15 calls, about $0.02); item-level parsing in P190. P189 was built with `deepseek-flash` as the default model (the founder, 2026-11-12) | Founder, before P190 | Entry 47 |
| 38 | **Closed 2026-11-12 (Entry 51): the founder said yes; done in P189b.** Was: **A rollback to V6.1 after V6.2's migration would not boot.** sqlx refuses a database holding a migration the binary does not know (`VersionMissing(2)`, reproduced locally). Recommended: (a) V6.2's `db::migrate` sets `ignore_missing(true)`, so later rollbacks (V6.3 → V6.2) work; and (b) §14.4 and the merge's rollback note gain one step: before booting V6.1 on a migrated database, `DELETE FROM _sqlx_migrations WHERE version = 2` (the three tables stay, unused). Nothing was changed; the founder decides, before P212 | Founder, before P212 | Entry 48 |
| 39 | **Checkpoint 3's points, open after the founder's go (2026-11-13, no change asked).** Each is the founder's, for the quality pass (BP §8A.12) or earlier: (1) Clean: a highlight looks like the spoken word's pill; (2) Bold, Mono, Glow: a coloured word is the spoken word's colour; (3) Mono: the cursor joins a spoken word's highlight; (4) Bold: a highlight close to a popping neighbour; (5) a punch word followed at once by speech shows about 0.4-0.5 s; (6) Glow's halo and Mono's cursor grow with a punch word; (7) at Bottom an emoji "under" its word is drawn over it; (8) white icons on a yellow disc are weak in contrast (**done in P206b, Entry 70**); (9) at Bottom the icon sits about 120 lp above the captions; (10) the hook is centred, over the speaker's face; (11) **the landing page has no sign-in entry, yet AI plans need one** (**done in P206a, Entry 69**); (12) the model may leave the hook item out (it did on the founder's clip) (**done in P206c, Entry 71**). The founder asked for (8), (11) and (12) to be fixed (2026-11-13, Entry 68); the others stay open | Founder | Entries 63-67, `~/v6.2-review/look/README.md` |

## Known issues for later prompts

| # | Issue | Prompt |
|---|---|---|
| 3 | Carried from V5 (its known issue 3): `openEditor`'s wait for the drop zone fails on the hosted runner, most often on the first model case after `cancel.spec.ts`. Run 41502704869 failed it and its retry; a re-run passed. Its message showed `UNSUPPORTED_STORAGE`, so the storage check now has a second try (Entry 12). Whether that cures the flake is read on the next CI runs: the first attempt's page held the drop zone's words only as the 90 s ended, which a storage retry may not explain. Push 2's run 41602738386 and push 3's run 41822657804: nothing flaky. **The merge's run 41827888094 failed it on both tries** (`model-download.spec.ts`'s first case, right after `cancel.spec.ts`): the first try held "Checking your browser…" for 90 s, the retry read `UNSUPPORTED_WEBGPU`, and the setup's own check read `UNSUPPORTED_WEBGPU` once that run. A re-run of the job passed. Suspects: the runner's WebGPU adapter coming and going, and `start-app.ts`'s `anonymousId()`, an IndexedDB read with no timeout before the capability check (since V1). **Seen again (V6.2 push 2, run 43263410381):** two `model-download.spec.ts` `@no-gpu` cases held "Checking your browser…" and passed on their retry; the run was green | V6.1 |
| 4 | **Done in P192 (Entry 54): `requestPlan` makes one attempt (`retry: "none"`), as `transcribeAudio` does; `http.ts` is unchanged.** Was: (P191) `web/src/net/http.ts` reads a 502, 503 or 504 as a waking API and tries again, unless its body's code is in `PROVIDER_FAILURES`. `plan_unavailable` (502, 504) and `plan_invalid` (502) are answers from a running API: `request-plan.ts` must not wait them out as a cold start. Add them to `PROVIDER_FAILURES` with their own `ErrorCode`, or call `/plan` without the waking retry | P192 |
| 6 | **Done in P193 (Entry 55): (a) `import-clip.ts` clears `plans` behind the flag; (b) a cancelled run sends no `plan_result`.** Was: (P192) Two things the pipeline's wiring must do. (a) `plans-repo.ts`' `clear()` is called where `import-clip.ts` sweeps the clip before (D-40), only with `ai_plan` on, so a past clip's plan does not stay behind. (b) `planFor` reads a cancel by the caller as `unavailable`: `plan_result` is not sent when the run's own signal was aborted, as `transcribe-cloud.ts` does | P193 |
| 7 | **Done in P194 (Entry 56).** Was: (P193) The plan's interface, with its copy in `copy/plan.ts`: the settings row "AI edit plan" calling `chooseAiPlan` (its help line names DeepSeek, open item 35, wording at CP 3); the "Regenerate plan" control calling `regeneratePlan` and its toast for each reason (`plan.regenerate.failed` for `busy` and `failed`); the fallback line, once, from the clip's `planFallback`, the sign-in line for `signed_out` (D-494). The logic is done and tested; nothing shows it yet | P194 |
| 8 | **Moved to P211 by P210 (Entry 76):** the fake API answers at once, so no wait can be read here; P211 reads the real API's wait and the founder judges it at CP 4. Was: (P194) `plan.processing` ("Planning the edit") is not shown: the plan's wait sits inside the processing card's "Finding visual moments" step, and the store has no field that says a request is out. Show it with the flags-on run of P210 if the wait reads as a stall | P211 |
| 9 | **Done in P196 (Entry 58): no dark fringe at 0.5×, 1.37× or 1.5×, read as pixels; Vello premultiplies each texel before it filters.** Was: (P195) Every transparent pixel of the bundle is 0, 0, 0, 0 (straight alpha). P183 read no dark fringe with Vello's straight-alpha image fill on a test disc; `image_basic` must show none on the real files at 0.5×, 1× and 1.5× and at opacity 0.5, read at 200% | P196 |
| 10 | (P199) `flags-identity.spec.ts` line 130, `expect(on.version).toBe(2)`, fails since P192: with `ai_plan` on, the flags-on build opens the `plans` store at structure version 3 (D-469), as designed. Every other check of the case passes with the pin read as 3 (P199's temporary probe: the same records, the same export). Widen it with D-440 (8)'s edit of the spec: version 3 when the build has `ai_plan`, else 2, declared | P209 |
| 11 | **The lavapipe SIGSEGV in `golden_frames`** (carried from V5 CL known issue 9 and the cloud release's known issue 2): the test process dies on CI's software GPU partway through the goldens, after the ones read so far passed. Seen in V6.2 at push 1 (run 43185400050 on `6078648`, a server-only commit; not read at the time) and push 3 (run 43329295884 on `1c94d83`, attempt 1: 14 of 33 goldens passed, each 0.9997, then the crash; attempt 2 passed in full). The rule stands: read the log, re-run the failed job once. **Two of V6.2's three pushes so far**, where V5 saw one in 26 runs: if it stops a re-run too, or keeps this rate past push 4, look into it (one thread per golden, or a fresh device per test) | Watch |
| 5 | (P191) §14.2's deploy check sends `POST /api/v1/plan` with a tiny body and expects `E_PLAN_UNAVAILABLE`; since D-494 it answers 401 without a bearer, before the switch is read. Corrected in the plan's §14.2 | P212 |

## Entry 0 - 2026-11-02 - Spec revision v1.3: the long path (outside a prompt)

**Founder's decisions.**

| Question | Decision |
|---|---|
| Order | V6 first, then V6.1-V6.11, then V7-V10 |
| The rule | Nothing that works is broken without being certain the new thing works; each release leaves the code ready for the next; a feature not worth what it would break goes back to the founder |
| Old V6.x items kept | Emoji, video inside shapes and shape transitions, all cards |
| Moved post-launch | Chapter markers, lower third |
| Person mask | Text behind the speaker and background change both in V6.11, each gated on quality |
| AI cost | A paid provider is allowed under a monthly cap; everything else stays on free tiers |
| Free plan | 1 long video a month, with all its shorts in preview; 3 exports a month as before |
| Vision | Local signals only; no frame leaves the computer |
| Test week (E-12) | Optional. Recommended and written down as such; the build does not wait for it |
| Positioning | Long videos and short clips equal on the landing page |
| B-roll | The user's own files only at launch |
| Tooscut | Read to learn approaches; Offcut's code is written here. No copying or line-by-line translation (Elastic License 2.0) |

**Decisions made while writing, for the founder to overrule.**

| Decision | Why |
|---|---|
| A short is a **virtual clip** behind an `Assembly` | The whole downstream pipeline stays as it is, and moving a hook to the front costs nothing. `TimeMap` keeps its ascending contract |
| The Director selects **whole sentences**, with a sentence-level request | Mid-sentence cuts cannot happen, by type, and a 20-minute request fits INV-4's 64 kB |
| A long source is **copied to OPFS** | Every worker keeps its handle, and no file name is stored through a `FileSystemFileHandle` (P-11). Cost: disk; TE-17 reads it |
| Long videos **need an account at the drop** | Costs real money, and a limit must be shown before the work. Clips stay anonymous |
| Offline **rules producer** for a shorts list | A failed AI request never dead-ends a long job |
| Every inference model runs in `asr.worker` | The worker count stays four, and ONNX Runtime stays one runtime |
| Person matte stored as a **low-resolution H.264 video** | INV-12 holds, and playback needs no new code |
| Chat returns **typed `PlanOp`s** in `EditState` | The validator stays the only gate, and undo is the existing history |
| `plan_usage` becomes `ai_usage` with a `kind` | One table for plan, shorts and chat; it was never created |
| Density rule V-9 applies to **model plans only** | Applied to rules plans, it would have changed V5's frames |
| Creator stays at $15 | Founder has not changed it; E-5 reads it |

**Changed.**

| File | What |
|---|---|
| `docs/technicalspec.md` (v1.3) | §41 compatibility contract; §42 version map; §43 long source; §44 `Assembly`; §45 Director; §46 vision; §47 launch catalog; §48 layers; §49 chat; §50 brand, memory, queue; §51 person mask; §52 AI cost; §53 experiments TE-17 to TE-25, INV-25 to INV-31, budgets, study notes, analytics, open items. §0, §5 tree, §10, §15, §16, §22.1, §23, §25.1, §28, §36, §37, §38 (TDR-22 to TDR-25), §39.3, §40 updated. All old V6.x references renumbered |
| `docs/product.md` (v1.3) | Positioning, ICP, a new persona, jobs 7-10, differentiation with an honest table of where Offcut can win and the moat, principles 3, 4, 6, 7; §9.1, §9.2, §9.4, §9.5, §9.6-§9.8 (H5); §10.1 long-path journey; §11; §12.7, §12.8, new §12.9-§12.15; §13.1, new §13.9 with the 3,000-paid-user arithmetic and channels; §14-§18; §19 E-12 to E-14; §20.1, new §20.2A; §21 items 15-21 |
| `docs/buildplan.md` (v1.3) | Header, version table, rules; V6's first prompt and carried items; §8A rewritten: rules for all releases, founder's work, estimate, then goal, demo, dependencies, build list, experiments and exit boxes for each of V6.1-V6.11; additions to V7-V10; Appendices A-D |
| `docs/v6/v6changelog.md` | New: this file |

**Learned from studying Tooscut, in our own design.** Windowed region decoding of audio; per-frame source requests grouped by source; layers in a frame description; cross-transition boundaries; the HE-AAC sample-rate correction. Not adopted: text drawn last above all layers, a 169-sample blur, preview and export through different decoders. Recorded in TS §53.4.

**Checked.** Documents only: no code or test changed, and nothing was built. `node scripts/check-file-tree.mjs` passes on the revised tree. Every `§41`-`§53` reference inside the new sections, every `TS §` reference from PS and BP, and the new PS section references resolve to a real heading. Anchors for every in-place edit matched exactly once, or the patch stopped.

**Estimates, not readings.** Time and cost figures for the long path (21-27 minutes of ASR on D1, 25-55 minutes end to end, 0.7-3.5 cents of AI per long video, 143 prompts, February 2027) are derived from V5's numbers and third-party price listings. TE-15, TE-18 and V9 read them. Competitor statements come from third-party sources or are unconfirmed, and are marked so in PS; E-12 tests the main one by hand.

**Not done.** Not committed: the founder asked for the documents. `docs/v6/v6implementation.md`, the V6 build guide and prompts come next, as for earlier versions. The 57 code comments stay until V6's first prompt.

## Entry 0.1 - 2026-11-02 - Mail by SMTP with lettre; merchant under review (outside a prompt)

**Founder's decisions.** Email goes out through `lettre` 0.11 over SMTP. The merchant application is submitted: the dashboard shows personal information approved, product and bank details recorded, review stated as up to 72 hours, and test mode available.

**Checks made (scratch project outside the repository; nothing here was built into the product).**

| Check | Result |
|---|---|
| The features as first proposed, `["smtp-transport", "builder"]` | Default features stay on, so they bring `native-tls`, where the server uses rustls, and no `tokio1`, so sending would block an async handler |
| `default-features = false`, features `smtp-transport`, `builder`, `hostname`, `tokio1-rustls`, `ring`, `webpki-roots` | `cargo check` passes; no `native-tls` or OpenSSL in the tree; `rustls` 0.23.46 |
| `cargo deny check licenses bans` with this repository's `deny.toml` | Bans pass. One licence error: `quoted_printable` 0.5.2 is `0BSD`, which needs one line in `allow` |
| Render's free web services | Render's changelog: outbound 25, 465 and 587 blocked since 2025-09-26. Port 2525 is not on that list. **Not tried from Render** |

**To change when V6's plan is written.** TS §5 and §22.8 and BP §8.1 name `http_mailer.rs` and `MAIL_API_KEY`. They become `smtp_mailer.rs` and `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, with `MAIL_FROM` kept, in `config.rs` and `render.yaml`. `config.rs` aborts on a missing variable, so the new values go into Render before the deploy that reads them. `0BSD` goes into `deny.toml` in the same change as the dependency.

**Brevo, same day.** Account made; SMTP login with a key on port 2525 succeeds from D1, with no email sent. This does not prove Render can reach it, or delivery. Brevo flags the sender: a Gmail address, DKIM "Default" and DMARC "Freemail domain is not recommended". The first real sends read whether mail arrives (TE-8). The SMTP key expires on 2027-11-02.

**Open.** Open items 9 and 10. If no port works from Render, the `Mailer` trait keeps the change to one file: an HTTP adapter without `lettre`, or a paid Render instance, which is the founder's decision (BP §1.1).

## Entry 0.2 - 2026-11-02 - V6's plan, guide and prompts (outside a prompt)

**Written.** `v6implementation.md` (D-241 to D-292), `v6buildguide.md`, `coding-promptsv6.md` (Setup and P137 to P150), after reading the code at `3cae48e` and the specs. Fourteen prompts against BP's rough 10.

**Checked.** Documents only: no product code changed and nothing was deployed. One test mail through Brevo was accepted, to the sender's own address. In a scratch project, with this repository's `deny.toml`: `lettre`, `reqwest` with `rustls-no-provider`, `rustls` on `ring`, `hmac`, `getrandom` and `sqlx` compile together on one crypto provider, STARTTLS on port 2525 and HTTPS connect; `cargo deny` reports only the two allowances the plan makes. The port probe's command runs on D1. Not run: anything on Render, any Dodo call with a key.

**Found in the specs while planning, each decided in the plan.** The spec's `parse_event` cannot see Dodo's event id (D-274). Refresh in the grace with no new cookie would sign people out after a lost response (D-258). `http.ts` reads the server's own 502 and 503 as a waking API (D-287). `magic_links` has no column for the address as given (D-260). "Minus outbox length" would spend Free counts on Creator exports and break a protected suite (D-267). A webhook for a just-deleted user would fail its foreign key and retry eight times (D-272). Recovery from a failed renewal has no row in the table (D-276). Changing one protected assertion needs a new row, DC-12, for the founder (D-246).

**Dodo's documents do not say,** and P144 reads in test mode before writing the mapping (D-279): whether checkout `metadata` reaches subscription webhooks, the refund event's name and fields, which event a scheduled cancel sends, whether `webhook-id` is stable across retries.

**Changes Entry 0.1's plan.** TS and BP are copied back in P150 (TS §5 as files appear), as in V5, not when the plan is written. Decisions bind meanwhile (the SAB's precedence).

**Open.** Open items 9, 10, 11, 12.

## Entry 1 - 2026-11-02 - Setup: the branch, the documents, the floor

**Founder's answers.** The merchant is Dodo Payments. Mail test 1 arrived; the answer does not say inbox or spam (open item 12). DC-12 is yes, "but carefully": the plan now fixes it to three lines of one case, with the rest of that case and file untouched, and P147 quotes the diff (D-246, TS §41.2).

**Render's port check is not done.** The founder asked for it to be run, and the agent cannot: no Render API key exists here, and the deploy hook of `.env.deploy` deploys production, so it is not a way to run a test. It stays the founder's step, G 2.1, and blocks P138.

**Done.** `v6-build` from `v5-build` at `3cae48e`, which is `origin/main` (`4ff3b74`) plus that one docs commit. The v1.3 specs and V6's documents are this commit. TS §5 has the `docs/v6/` rows; TS §41.2 has DC-12.

**Counts.** Read on D1 on this tree: Rust 657 passed, 0 failed, 4 ignored; Vitest 336 passed. Both equal the floor (D-244). `pnpm e2e`, the verifier self-test and the media suite were not re-run: no code has changed since V5's close.

**After the commit, same day.** Mail test 1 reached the inbox (founder). It went from a Gmail address to itself, so it says little about other mailboxes; TE-8 reads those. The first port probe failed on Render with "Application exited early" and no output: the command held quotes and spaces, and Render does not document how that field parses them. A second form, with no quote and no space, puts the script in an environment variable, and it ran.

**Render's reading (founder's logs, 2026-11-02, free web service):** `2525: open`, `587: blocked`, `465: blocked`. `lettre` stays (D-252, V6 section 6.2). This is a TCP connection only, so open item 9 stays until a real mail goes out from Render.

## Entry 2 - 2026-11-02 - The audit of the V6 documents: fixes before P137

**Source.** An outside audit the founder supplied: 9 blocking problems, 7 gaps, smaller items and stale text. Each was checked against the code at `v6-build` before being applied. All held. One was narrower than reported: Dodo's documents do give a way to force a test renewal, by patching `next_billing_date`, with a failure card, so item 8 needs no short-interval product.

**Founder's decision.** The founding offer goes live after V6.6 (D-303). BP §8A.6 now holds a founding release, with live payments, every offer purchasable, the legal pages moved forward, and Vercel's terms accepted. BP §8.2's TE-9 row, §11.3, §12.3 and PS §19 follow. V6 itself ships it off.

**The plan was wrong on sessions, and now is not.** D-258 as first written moved the previous hash on each use and retried refresh for 70 s against a 30 s grace. It would have signed people out in three ways: a third request with one cookie, a late retry read as reuse, and crossed responses read as reuse 15 minutes later. D-258 is rewritten; the lock and the 25 s budget are D-297; the fourth row of §7.3 sends no `Set-Cookie`; the tests now hold three concurrent refreshes and a rotated-away token used after 30 s.

**Fixed.**

| Audit item | Decision |
|---|---|
| Two more protected assertions (lines 142, 321) | D-293: the title keeps the sentence, so neither changes and no new row |
| Seeded tokens | D-294 |
| Main sign-in path illegal | D-295 |
| Callback tab races itself | D-296 |
| Prompts out of order | D-298 |
| DC-11 fails the frozen check | D-245, G 0.4: `verify/README.md` excluded by name |
| P144 cannot answer its questions | G Phase 8 rewritten, with a logging endpoint (V6 §6.3) and forced renewals |
| M1 walk cannot finish | G 13: local rows with the key set-up; checkout at G 14.6 |
| Old subscription's events; double charge | D-299, D-300 |
| Protected interface limits | D-302, named in P147's and P148's gates |
| Receipts across accounts | D-267: `sub` on the row |
| Mail proven too late | D-304: a real send from Render and TE-8 in P138; visible From recorded; rollback in §18 |
| S-10 half built | D-270: an export test against the list |
| Unlisted test edit | D-246 names `start-export.test.ts`'s case |
| Smaller items | Close line before 134 (D-246); `Resumed`, `PlanChanged` (D-301); `is_partial` (D-277); `forCheckout` needs a session (§12); open the link on this computer (D-302) |
| Stale text and core-doc conflicts | §1A, D-242, D-260, D-305, §4's count, the channel's name, §21's additions, G 14.2, G 14.6, the "feature-complete" wording |

**Corrected for the record.** Entry 1's open-item numbers: item 11 closed with DC-12's yes, item 12 with the inbox reading, item 13 with pull request 10. Entry 1's "Docker" assumption in the older documents: server tests need Postgres on port 9000 only (D-305).

**Tested.** The two new probes ran on D1 as plain argument lists: the webhook log answered 500, 200 and 500 for a first delivery, a retry and another event, and served its log; the mail probe sent "Offcut mail test 2" to the sender's own address. Not run: anything on Render, any call to Dodo with a key.

**Counted once.** Thirteen new decisions make D-241 to D-305. Fourteen prompts stay; P138 and P144 grow.

## Entry 3 - 2026-11-02 - Pricing and legal pages, ahead of V6 (outside a prompt)

**Founder's decisions.** Pricing, Terms, Privacy, Refund and contact go live now, since Dodo's FAQ says its review reads them on the live site, and there is little harm in showing them. Dodo's form holds no price, so the site shows the plan's: Free $0, Creator $15/month or $144/year. The contact is the Brevo sender's address. The pages name "Offcut" only, for now.

**Built on `site-pages`, from `origin/main`,** not on this branch: V6 will not merge for days. A section on the landing page with no way to buy, saying "not on sale yet" and listing only export limits; `config/pricing.ts`; `ui/pages/LegalPage.tsx` at `/legal/:doc`, both already in TS §5; a footer; all copy in `messages.ts`. No request, no event, no change to the clip path, so the exported file is untouched. This branch takes it by merging `origin/main` once the founder has merged it, before P146 touches `routes.tsx` and `messages.ts`.

**What this moves.** BP §8A.6's first box is done early, apart from a re-read when accounts open. Those pages say what happens "when accounts open", so P148 re-reads them against what V6 built.

**Read on D1.** `pnpm check` exit 0; Vitest 336; `pnpm e2e` 16 passed, up from 12; the new pages looked at. CI not read: it needs the founder's pull request. **Not lawyer-reviewed:** BP §10.3's known gap. The founder reads them before merging.

## Entry 4 - 2026-11-02 - P137: comments through the version map; known issues 10 and 3

**Before it.** The founder read the legal pages and merged pull request 11 as `343a7c9`. This branch merged `origin/main` as `1f8d054`, ahead of the plan's "before P146", so V6 is built on what is live.

**DC-11.** One regex, one pass (D-245): 49 hits read, 48 lines changed in 24 files, the regenerated `web/src/gen/domain.ts` among them. Two sentences the map alone made wrong were corrected by hand: chapters now read post-launch, and `ChangeSummary`'s comment no longer names a chapters row in V6.6. `verify/README.md` line 16 by name. **Proof:** a script stripped comments from each of the 23 code files at `HEAD` and in the tree, strings left alone, and they compared equal. No `.snap` or `.png` in the diff. The renumbered comments were read once as sentences. TS §42 said about 57 comments; 49 lines is what exists.

**Known issue 10, closed.** `aria-details` is set only on the Captions tab, where the transcript exists. `review-edit`'s case 16 now scans all four tabs: on the old `PreviewPlayer.tsx` it failed with `aria-valid-attr-value: 1 node(s)` on the Style tab, on the new it passes, keyed build, `d1`.

**Known issue 3, read and not cured (30 minutes, as the prompt limits).** V5's last run (41289198237) shows `getByTestId('drop-zone')` not found for 90 s, then a pass on the retry, with nothing saying what the page held. Whether the fake clock holds the capability check back cannot be told from that, and a change made on a guess would hide it. `openEditor` in `model-download.spec.ts` now throws with the page's text, so the next occurrence names the cause. The wait is unchanged.

**A wrong turn, for the record.** A first keyed build took the wrong value for the test key, from a greedy `sed` over `ci.yml`, so the app stopped at start with no drop zone. It looked like known issue 3 and was not. Nothing committed came of it.

**Counts.** Rust 657 / 0 / 4; Vitest 336; `pnpm check` passes from the commit (`check-gen-clean` compares with what is committed). Frozen check: no output. `pnpm e2e` not re-run: nothing it reads changed since Entry 3's 16.

## Entry 5 - 2026-11-02 - The pages went live; Vercel blocks a private repository's deploy (outside a prompt)

**What happened.** The founder made the repository private on 2026-11-01. Pull request 11's merge `343a7c9` passed `ci`, `e2e-media` and `deploy-api`, then `deploy-web` hung at `vercel deploy --prebuilt --prod`, which took 5 s the day before. Vercel's dashboard said: "The deployment was blocked because the commit email 133807457+mayanpathak@users.noreply.github.com could not be matched to a GitHub account." Vercel's Redeploy button was not used: it would skip the test-key check.

**Fixed for now, by the founder's decision.** Before anything was published, every commit on every pushed branch was searched for the real values of `.env.deploy`: none found, and no `.env`, key or video was ever committed. The agent made the repository public and re-ran only `deploy-web`. Attempt 2 passed every step, including the rewrite, the headers and `@smoke`. The live bundle holds the new pages; `healthz` shows `343a7c9`.

**What this proves and does not.** The same commit and email deploy while public and are blocked while private, so visibility decides it today. It does not say why: every merge on `main` since 2026-10-07 has this same noreply address, and the founder recalls deploys working while private around V3. Vercel may have tightened the check, which its forum suggests; nobody here can read that from its side.

**Going private again.** The next merge to `main` will meet the same block, and private Actions minutes ran out once already (2026-10-29), at about 50-75 minutes a push. `vercel-email-check` holds one docs commit, ready to test merging with the Gmail address while private. That is untested.

## Entry 6 - 2026-11-02 - P138: mail through lettre, config, fakes

**Changed.** `mail/{mod,smtp_mailer,templates}.rs`: the `Mailer` trait, `lettre` over STARTTLS with an 8 s limit, and one template with one link, no image and no tracking. `billing/{mod,provider}.rs`: the trait of V6 section 10.1 with `UnconfiguredBilling` (D-298). `AppState` holds both; `main.rs` installs `ring` once and builds them. `Config`: `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `BILLING_API_BASE_URL`; `MAIL_API_KEY` gone, also from `render.yaml`; `.env` names changed, 19 now, with placeholder values. `tests/common/mod.rs`: `FakeMailer`, `FakeBilling`. `deny.toml`: `0BSD`. TS §5: `smtp_mailer.rs`.

**Differs from the plan.**

| Point | What and why |
|---|---|
| Dependencies | Only what this prompt uses: `lettre`, `rustls`, `async-trait`. `getrandom` comes with P139, and `reqwest`, `hmac`, `webpki-roots` with P144, each with its `deny.toml` line and its re-read of `cargo tree` |
| `MAIL_FROM` | `Config` also checks that it parses as an address, so a bad value stops the boot by name |
| `SMTP_USERNAME` | Redacted in `Debug`, as a login is half a credential |
| `FakeBilling` | `verify_signature` and `parse_event` are stubs until P143 |
| Fakes as plain data | Public fields and one read in `TestApp::drop`: the repository has no `#[allow(dead_code)]`, and each test binary uses a different part of `common` |

**Read on D1.** `cargo deny check`: advisories, bans, licenses, sources ok. `cargo tree -p offcut-api` holds no `native-tls`, `openssl` or `aws-lc`. `Config` tests: each of the 19 missing or empty names itself, `SMTP_PORT` malformed names itself, no value in an error, 7 redactions in `Debug`. Mailer tests: nothing listening and a server that never answers are each `Unavailable`. **One real mail through the real `SmtpMailer`**, from a temporary example since deleted, on port 2525 at 16:25 UTC: `Ok`, to `MAIL_FROM`.

**Counts.** Rust 666 / 0 / 4, up from 657; Vitest 336; `pnpm check` exit 0; frozen check no output. Not pushed: the first push is P142.

**Open: the three human boxes of open item 9.** Not ticked. Until they are, nothing proves mail from Render with a login, or delivery beyond one Gmail address.

## Entry 7 - 2026-11-02 - P139: a link becomes a user, a session and a token

**P138's human boxes.** One real send from Render read `sent` in its logs (founder's screenshot, 17:07 UTC), so `lettre` on 2525 stands. **TE-8 is cut down by the founder's decision:** two Gmail addresses only, "if it works it works", no 50 mails. Three template mails went to the second address. `docs/v6/experiments.md` holds what this does not prove: no other mail provider has been tried. P138's last box stays with open item 9.

**Changed.** `offcut-entitlement`: `sign.rs` behind the feature `sign`, `tests/token_roundtrip.rs`, `tests/profile.rs`. `auth/{mod,session,access_token}.rs`; `db/{users,magic_links,sessions,subscriptions,usage_receipts}.rs`; `entitlement/issue.rs` as TS §22.3; `AppError`'s seven variants, mapped as D-264. Routes `POST /auth/magic-link` and `POST /auth/verify`, 1 kB each. `deny.toml`: `offcut-api` may use `getrandom` (D-257). `server/.sqlx/` gains 8 queries.

**Differs from the plan.**

| Point | What and why |
|---|---|
| `limit()` in `rate_limit.rs` | Not added. The one route here that needs a key from its body calls `limiter.check` directly, as `by_ip` does. P140 adds `limit()` with its first three users, so one helper is not written for one caller |
| `reqwest` in tests | Its `json` feature is on for the dev-dependency, for `auth_flow.rs` |
| `token_roundtrip.rs` | `required-features = ["sign"]`. It runs in `cargo test --workspace`, as CI does; `cargo test -p offcut-entitlement` alone needs `--features sign` |
| The access token's `verify` | Written and tested now, since verify's answer must be checked. The extractor that uses it is P140's |

**Read on D1.** `auth_flow.rs` 11 cases, among them: only the hash stored, a second request spends the first link, mail failure 503 and no row, expired, used, unknown, **two racing verifies give one success**, one user across two sign-ins, the cookie's text exact, the access token's lifetime, a Free token that verifies with `ENTITLEMENT_SIGNING_KEY`'s public key. `cargo tree -p offcut-wasm-render -e features` holds no `sign`. `SQLX_OFFLINE=true cargo check -p offcut-api --tests` passes.

**Counts.** Rust 691 / 0 / 4, up from 666; Vitest 336; `pnpm check` exit 0; frozen check no output; no `native-tls`, `openssl` or `aws-lc`. Not pushed: the next push is P142.

**Found, not fixed.** A failed mail deletes the new link, but the address's older unused link was already spent in the same statement. So a mail failure can kill a link that was still in the inbox. Small: the person asks again. As §7.1 says.

## Entry 8 - 2026-11-02 - P140: rotation, logout, the extractor, /me

**P138's last box, closed.** The founder read the three mails: all in the inbox of the second Gmail address, each link's host `example.invalid`, so Brevo does not rewrite the link (D-255). The Render mail-probe service is deleted, and pull request 12 is closed. The visible From was not reported, so `experiments.md` keeps it as unread.

**Changed.** `auth/extract.rs`'s `AuthUser`, verifying with no database read, and putting the user's tag on the request's log line (TS §24.4). `auth/session.rs`: `decide`, a pure function of §7.3's table, `refresh`, `check_csrf`, `refresh_token_of`, `cleared_cookie`. `POST /auth/refresh`, `POST /auth/logout`, `GET /me`. `limit()` in `rate_limit.rs`, which the magic-link route now uses too (Entry 7's difference, closed). `db/sessions.rs`, `db/users.rs`'s `email`, `db/subscriptions.rs`'s `summary`. `issue()` returns the plan beside the token, so `/me` has no second reckoning of a plan (D-266).

**Tests first.** 17 of the 18 new `auth_flow.rs` cases failed before the code; the 18th, wrong method, holds by the router's fallback.

**The tests were tried against the bugs they guard, then restored.**

| Bug put back | Caught by |
|---|---|
| A retry moves the previous hash, the plan's first design | `row_2`, `three_concurrent`, `a_token_rotated_away` |
| A reuse does not revoke | `row_3` |
| A refused refresh clears the cookie | `row_4` (both), `a_token_rotated_away` |

The four race and grace cases then ran 12 times: no failure.

**What holds after this, and what does not.** The lock of D-297 arrives with P145; until then only these tests stand behind the concurrent case. A session ends 60 days after its sign-in however active it is, as section 7.3 assumes, so a daily user signs in again every 60 days. It is one line to make it slide, and a founder's call if that proves annoying. The cookie through Vercel's rewrite and a real `Origin` are first read at G 14.6.

**Counts.** Rust 713 / 0 / 4, up from 691; Vitest 336; `pnpm check` exit 0; frozen check no output; no `native-tls`, `openssl` or `aws-lc`. Not pushed: the next push is P142.

## Entry 9 - 2026-11-02 - P141: the token and receipts

**Founder's decision.** A session ends 60 days after its sign-in, however active the person is: kept as section 7.3 assumes (Entry 8's question).

**Changed.** `GET /entitlement`, `POST /usage/receipts`, `usage/mod.rs` with `FairUseLog` (D-269), which `AppState` holds. `issue.rs`: `plan_now`, so a receipt's `plan_at_export` and a token's plan come from one place; `utc_month` is public for the fair-use count. `db/usage_receipts.rs`: `record`, `creator_used_since`.

**Tests.** `entitlement_rules.rs` 12: 3, 2, 1, 0 and floored; the seven fields; only Free exports count, and only one's own; the first instant of a month and of a year, by a clock passed in; a UTC month read from another offset; active, past due and canceled each Creator to the second their period ends, and Free the next; a refund; both route refusals. `usage_receipts.rs` 8: the row under the plan of now; Creator spends no Free count; a repeat and another user's id change nothing; **over the limit accepted, with a token**; no session and a gone user; a bad body; fair use logged once, at 101 and not 100, with the user's tag.

**Differs from the plan.** None. D-268's server half is that a receipt needs a session: without one it is 401, and the client's outbox waits (P145).

**Counts.** Rust 734 / 0 / 4, up from 713; Vitest 336, as nothing under `web/` changed; `pnpm check` exit 0. Not pushed: the next push is P142.

## Entry 10 - 2026-11-03 - P142: account, S-10, retention, the guard tests

**Changed.** `POST /account/delete`: the merchant first, 502 and nothing deleted if it refuses, then one transaction; 204 and a cleared cookie, also for an account already gone. `GET /account/export`: the user, session times, the subscription's summary, receipts oldest first; no hash and no merchant id. `USER_TABLES` in `db/mod.rs` (S-10). `purge_once` also takes links over a day old, webhook events over 90 days and sessions revoked or expired. `db/webhook_events.rs`, so far its purge only.

**Tests.** `account_delete.rs` 9, `account_export.rs` 6, `rate_limit.rs` 3, `no_media_routes.rs` 3, `log_redaction.rs` 1, and one retention case in `analytics_allowlist.rs`.

- `no_media_routes.rs` reads its routes from `router.rs`, so a new route is walked with nobody remembering. A new POST route fails until it is named as reading a body or not. P143 and P144 meet this.
- `log_redaction.rs` runs at `DEBUG`, louder than production. It carries 18 markers through sign-in, refresh, `/me`, a receipt, the export, analytics, the waitlist, a failed mail, a bad body, an unmatched path and the delete. It also asserts that the request lines are there, with the route's template and the user's tag, so an empty log cannot pass. The webhook joins in P143.
- `rate_limit.rs` covers 13 of the 16 groups through the real routes. The three billing groups join in P143 and P144.

**The tests were tried against the bugs they guard, then restored.**

| Bug put back | Caught by |
|---|---|
| `usage_receipts` left out of `USER_TABLES` | `user_tables_names_every_table_with_a_user_id_column` |
| A handler logs the address | `no_flow_writes_a_secret_or_a_person_to_the_log` |
| The export leaves out the receipts | `a_row_of_every_listed_table_appears_in_the_export`, and the shape case |
| A delete that skips the merchant | `a_live_subscription_is_cancelled_at_the_merchant`, and the refusal case |

**Differs from the plan.** `users.email` serves as the normalized address in the delete, since D-260 makes them the same.

**Noted for P144.** A cancel that succeeded at the merchant, followed by a crash before the delete commits, leaves a row still `active` until the webhook lands. A retry then cancels a subscription already cancelled, so `adapter.rs` must answer that as a success. If it does not, the person cannot delete their account.

**Tier 3.** `pnpm check` exit 0; `pnpm fixtures` up to date; `pnpm test`: Rust 758 / 0 / 4, up from 734, Vitest 336; `pnpm e2e` 16; frozen check no output; no `native-tls`, `openssl` or `aws-lc`. **Push 1:** P137's CI passed earlier; this one carries P138 to P142, and P143 reads it first.

## Entry 11 - 2026-11-03 - P143: billing core on a fake provider

**Before it.** Push 1's CI (run 41502704869): media failed on known issue 3, its first attempt and its retry; a re-run of the failed job passed, with the same case flaky first. No web or CI file had changed since the run before. See the known issue.

**Changed.** `billing/webhook.rs`: signature, then one transaction with the dedupe insert and the change. `next()`, pure, is TS §22.5's table with D-276's and D-301's rows. `billing/offers.rs`: offer to product and back. `db/subscriptions.rs`: `for_update`, `upsert`, `user_of_customer`. `db/webhook_events.rs`: `record`. `db/users.rs`: `lock`, so a delete cannot take a user from under a change. `POST /billing/webhook`, 64 kB, by address. `FakeBilling` reads its webhook from a plain JSON body. `no_media_routes.rs`, `rate_limit.rs` and `log_redaction.rs` meet the route.

**Tests first.** `billing_events.rs` 10: **all 32 cells**, 8 kinds by 4 starting states, 17 applied and 15 ignored, against rows written by hand apart from the code; an ignored cell changes nothing. Then older events, the same second, another subscription (D-299), a refund naming no subscription, a customer-only owner (D-275), an unknown customer (D-284), missing fields. `webhook_idempotency.rs` 11: bad signature; applied, and the next token says Creator; replay; two deliveries at once; of no interest; unreadable; **a `Canceled` for a deleted user is 200 with the event kept**; **a forced failure in the change leaves no event row, and the retry applies it**; 64 kB read and one over refused.

**The tests were tried against the bugs they guard, then restored.**

| Bug put back | Caught by |
|---|---|
| Older events apply | `an_event_older_than_the_last_one_applied_is_ignored` |
| Another subscription's event applies (D-299) | `an_event_for_another_subscription_is_ignored_except_activated` |
| A recovery from past due is ignored (D-276) | `every_cell_of_the_table` |
| The event row commits before the change | `a_failed_apply_leaves_no_event_row_and_the_retry_applies_it` |

**Decided where the plan leaves it open.** `Canceled` and `PlanChanged` with no date keep the date there is: an event with no date must not end paid time. A signed body that cannot be read is 400, so the merchant retries while it is fixed; unknown types are 200 in `adapter.rs`.

**Counts.** Rust 782 / 0 / 4, up from 758; Vitest 336; `pnpm check` exit 0. Not pushed: the next push is P148.

## Entry 12 - 2026-11-03 - The storage check's second try (outside a prompt, founder's decision)

**Why.** Known issue 3's new message showed a capable browser told "Your browser will not let Offcut keep files". The check has 2 s, and no answer counted as a no. A person on a busy laptop could be told the same, wrongly, and go looking for a setting that is not there.

**Changed.** `platform/capability.ts`: `storageAnswer`. A check that has not answered is asked once more, and either try's answer counts. A no or a rejection is final at once. Only this check; the others keep one wait.

**Trade-off, accepted by the founder.** With a slow disk the run can take up to 4 s, past PS §9.3's 3 s. A normal run is unchanged. P150 copies this to TS §13.2.

**Tests first.** 8 cases in `capability.test.ts`; 4 failed before the code. Two existing cases used `storageOk` as their example of a check that hangs, and now use `aacEncode`, or leave storage out, with the same assertions: this is not a weakened test, as the storage check's own rule has its own cases now. One case caught a real bug in the first version, which read a rejection as no answer yet and asked again.

**Counts.** Rust 782 / 0 / 4; Vitest 344, up from 336; `pnpm e2e` 16; `pnpm check` exit 0.

## Entry 13 - 2026-11-03 - P144: the provider's adapter, from real payloads

**Founder's steps.** Three test-mode products, tax-inclusive (the founder's decision: the price on the page is what a person pays), an API key, and a throwaway logging endpoint for webhooks. Payments and a refund by hand. **Still the founder's:** delete the logger service on Render, and remove its webhook endpoint from Dodo's test mode (open item 16).

**What real deliveries answered, 64 of them (V6 §10.2's questions).**

| Question | Reading |
|---|---|
| Does `metadata.user_id` reach subscription events? | **Yes**, and payment events. D-280's stop is not met |
| Is `webhook-id` the same on a retry? | **Yes**: the same id with a new timestamp. No stop |
| Do real signatures verify? | **Yes**, 64 of 64 through the Rust adapter, with the real secret |
| A first payment | `subscription.active`, `renewed` and `updated` in one second, in either order |
| A renewal | `subscription.renewed`, with the new `next_billing_date` |
| A declined renewal | `payment.failed`, then `subscription.on_hold`. **`next_billing_date` has already moved forward**, so the paid period is read only from kinds that say a period was paid |
| Recovery | `renewed` and `active` together. D-276 is real, and either alone recovers |
| A scheduled cancel, and taking it back | `subscription.updated`, with its flag |
| A cancel | `subscription.cancelled` and `updated`, `next_billing_date` kept as the paid time |
| A refund | `refund.succeeded`: `is_partial`, the customer, **no metadata, no subscription id**. D-275 is real |
| A second cancel | 200, so a retried delete cannot stick (Entry 10's note) |
| A partial refund, `plan_changed`, `past_due` | Not produced. Mapped from the documents and fixtures edited to match. **G 14.6 reads them if they occur** |

**Two things only the real payloads showed, and the code changed for them.**

1. **A refund does not stop the subscription at the merchant.** With the plan as written, the row said `canceled` while Dodo went on charging monthly, the renewals ignored from `canceled`: a person paying and on Free. The webhook now cancels at the merchant, inside the transaction, so a failure leaves no event row and the retry does it. The Refund policy on the live site says a refund ends the plan, which is now true.
2. **`subscription.updated` fires on nearly every change.** Mapped to `Resumed`, 28 of 64 deliveries, each moved `last_event_at` and could have hidden a retried older event. `Resumed` now applies only to a row with a cancel scheduled. Entry 11's line saying otherwise is removed.

**Changed.** `billing/adapter.rs`; `POST /billing/checkout`, with D-300's refusal; `GET /billing/portal`; `db/subscriptions.rs`'s `customer_id`; `main.rs` builds the real provider, and a secret that is not a `whsec_` value stops the boot, so G 14.4 says so. `reqwest`, `hmac` join the server. `UnconfiguredBilling` goes: nothing uses it now.

**Differs from the plan.** `webpki-roots` is not added: the image has `ca-certificates`. Fixtures replace the real address and phone though the founder did not mind, because the repository is public and a fixture does not need them; the script checked 20 real values absent from all 11 files. `past_due` from `subscription.on_hold` is the same kind as `subscription.past_due`.

**Tests.** `dodo_adapter.rs` 18, including Standard Webhooks' published vector and one-byte, wrong-secret, old-timestamp and missing-header refusals. `billing_routes.rs` 8. `webhook_idempotency.rs` +2, on the refund's cancel. `billing_events.rs` +1. **Against the real thing, from a temporary program since deleted:** every real delivery parsed, with no error; a real checkout and portal link; a cancel of a cancelled subscription; a bad id and an unknown subscription each refused.

**The tests were tried against the bugs they guard, then restored.**

| Bug put back | Caught by |
|---|---|
| A refund does not cancel at the merchant | `a_refund_cancels_the_subscription_at_the_merchant`, and the retry case |
| The signature is not compared | The one-byte, wrong-secret and vector cases |
| A declined renewal moves the paid period | `subscription_on_hold_is_past_due_and_does_not_move_the_paid_period` |

**Open.** A refund names only the customer, so a refund of an old payment also ends the current plan. The founder issues refunds, so this is known and left. No live-mode call has been made.

**Counts.** Rust 812 / 0 / 4, up from 782; Vitest 344, as nothing under `web/` changed; `pnpm check` exit 0; no `native-tls`, `openssl` or `aws-lc`. Not pushed: the next push is P148.

## Entry 14 - 2026-11-03 - P145: the client's foundations

**Changed.** `http.ts`; `api-client.ts` (the 12 functions, `authed`, the single-flight refresh under the lock); `state/{auth-store,entitlement-store}.ts`, `state/machines/auth-machine.ts`; `entitlement/entitlement.ts`; `persistence/receipt-outbox.ts`, `entitlement-repo.clear`, `schema.ts`'s outbox value; `platform/broadcast.ts`; `usecases/restore-session.ts`, `start-app.ts`'s marker, `main.tsx`; `tests-e2e/helpers/fake-api.ts`, every route, `/auth/refresh` 401 by default. `config/pricing.ts` is unchanged: it came with the pricing pages (Entry 3). No interface yet, and `start-export.ts` is not touched: nothing writes a receipt or reads a blocker until P147.

**REOPENED CONTRACT `http.ts` (D-286, D-287).** `HttpRequest` gains `bearer`, `csrf`, `budgetMs`. A 5xx whose body is `email_unavailable` or `billing_unavailable` is one attempt and its own failure. Every other 502, 503 and 504 still reads as a waking API.

**Differs from the plan.**

| What | Why |
|---|---|
| D-306: `connectAccount`, and one lint edge | `net` and `persistence` import no store. It runs first, as the callback page can verify before `start-app.ts` reaches its marker |
| D-307: a 429 keeps a receipt | §11.5's rule would have dropped a row that can still succeed, handing back a Free export. Any 4xx with no readable body stays too |
| D-308: `main.tsx` tells `startApp` whether to restore | `ROUTES` lives in `routes.tsx`, which imports every page |
| `authed` with a token already replaced uses it, without refreshing | A caller whose 401 lands after another's refresh ended would otherwise start a second one, which the acceptance forbids |
| A flush stops at the first row that cannot go | The others would meet the same answer |
| `api-client.test.ts`, `restore-session.test.ts`, `entitlement-store.test.ts` | New rows of TS §5: §4 did not name them, and these are where D-288, D-289, D-291 and D-292 are proven |

**One test changed: `landing.spec.ts`, not a protected file (TS §41.1).** "A dropped file" waited for `/healthz` as the last request the start makes. The refresh now follows it, so the case raced. It now waits for `/auth/refresh`, and a refresh after that wait still fails the case, so it is no weaker. Three cases added: one refresh, after the wake, with no error; a page that still works when the refresh never answers; no refresh on the callback tab.

**A protected proof that could have raced, measured.** `pipeline-preview`'s "quiet processing" fails on any request after the drop but files and analytics, and the refresh leaves at the same point the drop zone appears. Six runs, three at once: the refresh leaves 23 to 65 ms after the zone shows, and the drop ends 586 to 656 ms after. Both slow together on a slower machine, so D-289's marker stays and the spec is unchanged, confirmed by its hash. If a run ever names `/auth/refresh` in that case, this is why.

**Acceptance.** `auth-machine.test.ts` walks `unknown`, verify, `signed_in`. A 502 `billing_unavailable` makes one attempt and gives `E_BILLING_UNAVAILABLE`. Two callers meeting 401 cause one refresh. The refresh takes the lock, waits for a lock held by another tab, and stops at 25 s counted from its send, with attempts at 0, 11 and 23 s. Keyed build, `pipeline-preview`: "quiet processing" and "preview before sign-in" pass; "preview before sign-in" passes again with a temporary change making `/auth/refresh` never answer, in the same 1.5 minutes, so nothing waits on it. That change is gone, by hash.

**The tests were tried against the bugs they guard, then restored: 18 of 18 caught.** The budget ignored; a provider's failure retried; no CSRF header; no bearer; refresh not single-flight; no lock; the 70 s budget; refresh before the wake; a late 401 refreshing again; a 429 dropping a row; a lost session clearing the token (D-291); another account's rows sent (D-267); Creator rows counted as Free; a timer with no session (D-268); every account subtracted; no floor; the callback path throwing; an undecodable token kept (D-292).

**Not proven yet.** The outbox's database is an in-memory map in `receipt-outbox.test.ts`: real IndexedDB is read by `export-free.spec.ts` in P147. Two real tabs and the broadcast are read by `signin.spec.ts` in P146.

**Counts.** Vitest 455, up from 344; `pnpm e2e` 19, up from 16; Rust 812 / 0 / 4, as nothing under `server/` changed; `pnpm check` exit 0. Frozen check empty. Not pushed: the next push is P148.

## Entry 15 - 2026-11-03 - P146: sign-in on screen

**Changed.** `ui/components/SignInDialog.tsx`, holding `SignInForm` and `SignInDialog`; `ui/pages/AuthCallbackPage.tsx`; `ui/pages/AccountPage.tsx`, minimal: the form, and sign-out once signed in; `routes.tsx` mounts both; `usecases/{request-magic-link,complete-sign-in,sign-out}.ts`; copy for the four `E_AUTH_*`, off `COPY_PENDING`; `signin.spec.ts`, in `non-media`. `API_WAKING` is `messages.api.waking`, as it has been since V1. `signin_step` is sent at its two places. Nothing opens the dialog until P147.

**Differs from the plan.**

| What | Why |
|---|---|
| `SignInForm` uses the waitlist form's classes, and `[data-theme="dark"] .signIn` maps their colours to the editor's | One form on both themes. The pages' light colours in the dark dialog would fail contrast |
| A sign-out whose `/auth/logout` fails changes nothing and says so | The cookie cannot be removed from the page, so the next start would sign the person back in. D-291 does not say. Telling them is the honest way round on a shared computer |
| A callback whose connection failed keeps the token in memory and offers Try again | Only a dead link needs a new one. The token stays out of the address |
| A link with no token at all reads as `E_AUTH_LINK_INVALID`, with no request | Nothing to verify |
| "The callback tab does not ask for a session" moved from `landing.spec.ts` to `signin.spec.ts` | The callback now has its own page, and the landing file's helper reads the landing page. Stricter there: it also reads that the token is in no analytics batch |
| `sign-in.test.ts`: a new row of TS §5 | The three use-cases' rules, as §4 names none for them |

**Not read here.** Axe on the dialog itself: it opens only on a clip, so `export-free.spec.ts` reads it, in P147. `signin.spec.ts` reads axe on the same form on `/account`, on the sent state, on the error, on the callback's three states and signed in.

**The tests were tried against the bugs they guard, in the browser, then restored: 3 of 3 caught.** The callback restoring too; the token leaving the address late; a broadcast not heard.

**Counts.** Vitest 468, up from 455; `pnpm e2e` 33, up from 19 (landing 18, signin 15); Rust 812 / 0 / 4; copy pending 7, down from 11. Not pushed: the next push is P148.

## Entry 16 - 2026-11-03 - P147: the gate, the counter, the receipt

**Changed.** `blockers.ts`: the last three codes, `forCheckout`, `disablesExport`, `effectiveNow`. `export-store.ts`: `gate`, `gateFailure`, `upgradeOffer`; `unavailable` and `setUnavailable` gone. `start-export.ts`: the gate before the export, with D-294's reload, and the receipt in V5's one place; `planOf` gone. `ExportButton.tsx`: the counter, D-290's split, the last-export dialog. `UpgradePrompt.tsx`, a stub with both placements and no way to buy until P148. `use-shortcuts.ts`. Copy: both `E_ENTITLEMENT_*` and the three blockers, off `COPY_PENDING`, which now holds the two billing codes; `messages.export.unavailable` gone. `export-free.spec.ts`, in `MEDIA_SUITES`.

**DC-12, carefully. The whole of `git diff -- web/tests-e2e/export-creator.spec.ts`:**

```diff
-  test("no token: the button says exporting needs an account, and nothing is exported", async () => {
+  test("no token: the sign-in dialog opens with the limits, and nothing is exported", async () => {
-    await expect(page.getByTestId("export-status")).toHaveText(messages.export.unavailable, { timeout: 30_000 });
+    await expect(page.getByTestId("sign-in-dialog").getByTestId("sign-in-limits")).toHaveText(/720p.*1080p/, { timeout: 30_000 });
+    await page.keyboard.press("Escape");
```

The title, line 127's assertion, and one line closing the dialog before line 134. No import changed. The other lines of the case are untouched and pass, and the next case, "a token signed with another key", passes after it. Lines 142 and 321 pass as they are (D-293): `E_ENTITLEMENT_INVALID`'s title and `messages.editor.failedStub` are one constant. The case at lines 274-282 passes with the prompt showing.

**D-246's one Vitest case, `start-export.test.ts`:** `store().unavailable` became `store().gate === "sign_in"`; its `idle` and no-calls assertions stay.

**One helper in that file changed, not named by D-246, and no assertion with it.** `tokenNaming` made a token holding only `{ plan }`. D-292 counts a token it cannot read as none, so every case would have read "no token". It now writes all seven claims. The cases read the same.

**Differs from the plan.**

| What | Why |
|---|---|
| D-309: `forExport(room)` asks only the room; one lint edge, `blockers.ts` to `entitlement.ts` | An export in `gating` is not sent back to the sign-in dialog. TS §12.5 asks `effective()` from a layer that could not import it |
| D-310: `gateFailure` | TS C-7 asks for the error card, but nothing began, and the table's only way to `failed` would announce an `export_started` |
| D-311: the shortcuts stand aside for a dialog | Space on a dialog's button would play the video. **The plan did not foresee this** |
| A session with no token reads as `B_ENTITLEMENT_EXPIRED` | It can fetch one. TS's table names only `exp < now` |
| The last-export dialog's button says "Export it", not "Export" | Playwright sees a button behind a modal, and protected cases press "Export" by its exact name |
| The prompt's offer is once per export, by its id | A hit shares its export's id, so a dismissed one stays dismissed. PS §11's "after the first watermarked export" narrows this; P148 decides, with the final prompt |

**Found by the protected tests while building.** Waiting on the gate left a gap before `start`: two quick clicks both passed, and the second threw. `start-export.test.ts`'s "a second click" caught it. The re-check V5 had at that spot is back. `dialog` text in the DC-12 line reads the limits by a pattern and not exact words, so that a price change does not need a new row of TS §41.2; `export-free.spec.ts` reads the exact words.

**Acceptance, on keyed builds.**

| Suite | Result |
|---|---|
| `export-free.spec.ts` | 10 of 10. Three exports, each all 11 checks and each refused as a Creator's; signed out, enabled, the click opens the dialog; axe on both dialogs and the prompt; the link in a second tab closes the dialog; the button's name still exactly "Export" with the counter showing; a receipt on each miss, none on a hit; the fourth renders nothing, and Download again still works |
| `export-creator`, `timeline-preserved`, `cancel` | 34 of 34, in full and not by named case: all three seed tokens |
| `pnpm e2e` on CI's build, hosts check | 33 of 33; exit 0 |

**The tests were tried against the bugs they guard, then restored: 20 of 20 caught.** Unit, 19: the account never asked; an expired token usable; signed-out expired asking for a renewal; the outbox not subtracted at the limit; Creator blocked at zero; every blocker disabling the button; the room re-asking the account; checkout taking a cached token; no reload (D-294); the second-click check gone; no receipt on a miss; the receipt's plan from the page's claims (INV-9); no flush; a failed write failing the export; no last-export question; asked again after yes; a renewal failure not shown; the offer on Creator; a dismissed offer returning. Browser, 1: the shortcuts' guard removed. An earlier version of the keys case typed into a field, where shortcuts already stand aside, and so proved nothing of it: the case now presses Space with no field focused.

**Not run here.** `pipeline-preview`, `preview-sync`, `review-edit`, `rejections`, `model-download`: they export nothing, and the one full media run is P150's (G 0.5). `ExportButton` and `use-shortcuts.ts` are on their pages, so that run reads them.

**Found, not fixed.** `messages.ts` is at 653 of 700 lines, with P148's copy still to come. The Git Bash `python` is MSYS's and has no NumPy, so `verify_mp4.py` needs Python 3.13 first on `PATH` on D1.

**Counts.** Vitest 500, up from 468; `pnpm e2e` 33; Rust 812 / 0 / 4; media `export-free` 10 new; copy pending 2, down from 7. Frozen check empty. Not pushed: the next push is P148.

## Entry 17 - 2026-11-03 - P148: upgrade, checkout, the account page

**Changed.** `usecases/{start-checkout,confirm-checkout,open-billing-portal,delete-account,export-account-data}.ts`. `UpgradePrompt.tsx`, finished: the two offers, "Make another clip" after an export, "Not now", and its line for a failure. `AccountPage.tsx`, whole: the plan and its counter, the offers or "Manage subscription", the return from checkout with its wait, the account's data, deletion behind a question, sign-out. `TopBar.tsx`'s "Account" link, its path handed down from `routes.tsx` through `EditorPage` and `EditorLayout`. `upgrade_prompt` `shown` and `dismissed` in `start-export.ts`, `clicked` in `start-checkout.ts`; `checkout_step`'s three steps. The last copy: both billing errors, the account page and the top bar's link. `COPY_PENDING` is empty. `fake-api.ts` refuses a checkout for a live subscription and answers the portal 404 with none, as the server does. `checkout.spec.ts`, in `non-media`. A development hook, `fakeUpgrade`, for the captures.

**The legal and pricing words, for the founder to read (open item 17).** Each "when accounts open" that V6's merge makes true now says so; what is about payment stays as it was, as Creator goes on sale only with the founding release (D-303):

| Where | Was | Now |
|---|---|---|
| Pricing section | "Not on sale yet. These plans start when accounts open: leave your email below to hear when they do." | "The Free plan comes with a free account. Creator is not on sale yet: leave your email below to hear when it is." |
| Terms, Plans and payment | "They start when accounts open, and until then nothing can be bought." | "The Free plan starts when you create an account. Creator is not on sale yet, and until it is, nothing can be bought." |
| Privacy, heading | "What we will collect when accounts open" | "What we collect when you have an account" |
| Privacy, that section | "You will be able to export this data or delete your account" | "You can …", and two facts the server keeps that it did not name: when each sign-in began and was last used, and a sign-in link for a day at most |
| Privacy, Who handles it | "When accounts open, Brevo will send sign-in emails" | "Brevo sends the sign-in emails, so it receives your email address" |
| Refunds, first heading | "Before accounts open" | "Before Creator goes on sale" |
| All three | "Last updated 2 November 2026." | "3 November 2026." |

**Differs from the plan.**

| What | Why |
|---|---|
| D-312: "Account" opens in a tab of its own | Leaving `/app` would let go of the clip; `restoreClip` is V7's |
| D-313: the analytics queue is sent before leaving for checkout | The `started` step did not arrive when left to the send as the page unloads |
| D-314: once per watermarked export, until dismissed; "Make another clip" after an export only | Settles P147's open reading of PS §11 |
| D-315: offers on Free with no live subscription; "Manage subscription" with any subscription row; D-300's refusal goes to the portal in `startCheckout` | One way out of the refusal, from the prompt or the page |
| The wait for Creator reads again after a failed read, and stops when the session has ended | TS C-11 does not say. A 429 is likely for a page asking every 2 s; a lost session will not mend itself |
| The offers' buttons say "$15/month" and "$144/year" | At 1280 the panel was 5 px too narrow for "Creator, $15/month" beside "Creator, $144/year". The sentence above each pair names the plan |
| `account.test.ts`: a new row of TS §5 | §4 names no test of the five use-cases |

**Acceptance.**

| Box | Result |
|---|---|
| `COPY_PENDING` empty; its check and the analytics tests | `check-copy-codes`: pending 0, every list in full. `analytics_allowlist.rs` and the client's tests pass; no event or field was added |
| `checkout.spec.ts` as §15.4 | 12 of 12, three times over: to the provider's page; the return asks every 2 s until Creator, the address cleaned and the device's token Creator; past the minute `E_BILLING_PENDING` with Check again (the page's clock moved forward); the cancel; the portal; the refusal to the portal; the provider down; the data file; deletion |
| `UpgradePrompt` | Its placement type has two members; non-modal; outside `export-outcome`; no focus. `export-free.spec.ts` reads all three, its four buttons in order, and its events |
| `export-creator`, "summary and the question", unchanged in Tier 2 | The whole file, as its cases are serial: 18 of 18 with the prompt showing. The file has no change since P147 |
| Axe at 0; keyboard | Axe on every account state, the question, the prompt. Tab alone reaches the yearly offer and Enter buys; Tab and Enter open the question, the focus is on keeping, Escape keeps, Tab and Enter delete |
| The legal pages and the pricing section | The table above |
| Tier 3 | `pnpm build`, then `pnpm check && pnpm fixtures && pnpm test`, `pnpm e2e`: all exit 0. `pnpm e2e` 45 of 45 (landing 18, signin 15, checkout 12) |

**The tests were tried against the bugs they guard, then restored by hash: 16 of 16 caught.** No session check; no portal on D-300's refusal; no send before leaving; the click not told; no wait between reads; a failed read ending the wait; an ended session waiting on; the token not kept; a shorter minute; a retry told as a return; a failed deletion clearing the device; a deletion keeping the token; the data file's address kept; shown on every press at the limit; dismissed under the wrong placement; a dismissed offer shown again. Two were missed at first: the failed read was a 500, which `http.ts` retries as a waking API, so the test now reads a 429; and a guard in `offerUpgrade` that no flow could reach is gone.

**Captures, for the founder's one look.** `~/v6-review/`, 32 pictures at 1280 x 800 and 1920 x 1080: the sign-in dialog and the sent state, the callback, the counter, the last-export question, the prompt after an export and at the limit, the account page signed out, on Free, on Creator, with a payment not confirmed, and the delete question; and the empty, ready, exporting and done states with the new link. One capture failed once on its own race: the clock moved forward while the session's first refresh was still out. It now waits for the signed-in page.

**Found, not fixed.**
- An offer chosen in the editor's prompt leaves the page for the merchant's, as TS C-11 says, and with it the clip, until V7's `restoreClip`. After an export the file is already downloaded; at the limit the edit is lost.
- From the merge until the founding release, the offers lead to Dodo's test mode (D-285) while the pages say Creator is not on sale (open item 17).
- `messages.ts` is at 679 of 700 lines.

**Counts.** Vitest 522, up from 500; `pnpm e2e` 45, up from 33; Rust 816 / 0 / 4, every `test result` line of `pnpm test` summed: no Rust file has changed since P144's 812, so the 4 are a difference of counting, not new tests; media `export-free` 13, up from 10; copy pending 0, down from 2. Frozen check empty. **Push 2**, not waited on.

## Entry 18 - 2026-11-04 - P149: TE-9, the M1 list's local rows

**Push 2's CI.** Run 41602738386 is green: `ci` (Vitest 522, `pnpm e2e` 45, every Rust test) and `e2e-media` (26 passed, 1 skipped, nothing flaky). The deploy jobs were skipped, as on any branch. Nothing to fix.

**Changed.** `docs/v6/experiments.md`'s TE-9 and this entry. No product code, test or fixture: no reading asked for one.

**TE-9, as far as test mode allows.** In `experiments.md`. In short: a test purchase made on D1 turned the next token Creator. To get there, Dodo's real deliveries were forwarded to the local API byte for byte, with Dodo's own signatures. Also read: a real `subscription.plan_changed` applied as D-301 says, a real partial refund that ends nothing (D-277), and a cancel that keeps Creator to the period end. Not producible: `subscription.past_due`, as Dodo sends `on_hold`. Left for G 14.6: the deployed site's own endpoint, a delivery to a sleeping host, and whether the portal offers a plan change.

**The M1 walk (BP §8.3), on D1.** Set up as G 13 says: the API on 8080 with `APP_ORIGIN=http://localhost:5173`, the test seed's key, and `.env.deploy`'s five mail values; the web app through `pnpm --filter web dev` with `ci.yml`'s public key, checked against the seed before start. The founder walked it. The local database records what the walk did:

| Row | Ticked by | Evidence |
|---|---|---|
| Capability detection and the unsupported page | Walk | One browser refused with `UNSUPPORTED_AAC_DECODE`; Chrome or Edge passed |
| Input probe and actionable rejections | Founder, on V1-V5's proof | Not exercised: no `clip_rejected` |
| One-time model download | Founder, on V1-V5's proof | Not exercised: this origin had the model, so no `model_download` |
| Local transcription with word timestamps | Walk | WebGPU, 77 s for a 79 s clip |
| Pattern detector | Walk, in part | One number and one keyword. Lists and from-to on V1-V5's proof |
| Voice cleanup and loudness, duration unchanged | Walk | -14.1 LUFS; both exports 78.933 s |
| Captions in Clean, Bold, Tech | Founder, on V4's proof | Clean in the walk; no style was picked |
| NumberReveal, ListReveal, FromTo, KeywordPop | Walk, in part | NumberReveal and KeywordPop. The others on V1-V5's proof |
| 9:16 framing | Walk, in part | A landscape clip cropped; the offset not moved, no portrait clip |
| Preview with synced audio | Walk | `preview_played` |
| Review UI | Founder, on V4's proof | No `review_action`: no word edited, no event turned off |
| Export: Creator 1080 x 1920, Free 720 x 1280 with watermark | Walk | Both files pass `verify_mp4.py`, all 11 checks |
| "What we changed" summary | Walk | Shown after each export (founder) |
| Account: magic link, entitlement cache, usage receipt | Walk | A real mail signed in; a `free` receipt, then a `creator` receipt |
| Checkout, Creator plan | Walk, locally | Guide G 13 leaves this to G 14.6; it could be walked here because of the forwarding. G 14.6 still walks it on the deployed site |
| Settings page "What leaves your device" | Founder, on V4's proof | Not exercised: it sends no event |
| Allowlisted analytics, 19 events | Walk, in part | 11 of 19 arrived. The 8 not seen need a rejection, a download, an edit, a failure, a cancel, an answer, or clearing local data. The allowlist tests pass; the formal proof is V8's |

**Differs from the plan.**

| What | Why |
|---|---|
| The local API also took `.env.deploy`'s six test-mode billing values, not only the five mail values G 13 names | The boot stops on a secret that is not `whsec_` (P144), and `.env` holds placeholders |
| The checkout row walked locally, through the forwarded deliveries | The logger of G 8.2 records the signature, so a real delivery could be replayed unchanged. G 14.6 still walks it on the deployed site |
| Rows the walk did not reach, ticked by the founder on earlier versions' proof | The founder's decision, 2026-11-04. Each is named in the table above |

**Afterwards.** The test subscription was cancelled through the API; no test subscription is running. The two servers and the forwarding script are stopped. Its scripts lived in the session's scratchpad, printing no key, address, name, id or link.

**Open.** Open item 10 now reads Dodo's answer of 2026-11-04: identity and bank approved; the product on hold, as the live site reads as a waitlist. Open item 16: the logger can go. Open item 17 is unchanged and is needed before P150's merge.

**Found, not fixed.**
- The test purchase was charged in INR (1,476.89), not 15 USD: Dodo shows a local currency to a buyer in India, while the pricing section names dollars. Read again in live mode before the founding release.
- With `do_not_bill`, a plan change from monthly to annual keeps the monthly cycle's date, so the row reads annual with a period end a month away. That is correct for what was paid. A portal change, if the portal offers one, may bill differently: G 14.6.

**Counts.** Unchanged from Entry 17, as no code changed: Vitest 522, `pnpm e2e` 45, Rust 816 / 0 / 4, media `export-free` 13. Frozen check empty. Not pushed: the next push is P150's.

## Entry 19 - 2026-11-04 - P150: copy-back, the final gate (before the merge)

**Changed.** The copy-back of v6implementation §21, every row:

| Where | What |
|---|---|
| TS §1, §2, §3, §33 | Mail through Brevo's SMTP relay on 2525 (D-248, D-252); Dodo Payments named; `reqwest` and `lettre` rows; test-mode production until the founding release (D-285); the five SMTP variables in Render |
| TS §7 | V6's edges: `start-app.ts` to `restore-session.ts` (D-306), `blockers.ts` to `entitlement.ts` (D-309); `getrandom` direct in `offcut-api` (D-257) |
| TS §9 | C-1 (D-289, D-296, D-306, D-308), C-7 (D-290, D-294, D-309 to D-311, D-314), C-8 (D-254, D-259, D-287, D-302), C-11 (D-312 to D-315), C-19 (headers, the owner, the refund's cancel) |
| TS §12.1-§12.3, §12.5 | `effective()` counts this account's Free rows (D-267); claims read without verifying (D-292); the auth machine's new rows (D-295); the button split and the room-only gate (D-290, D-309, D-310) |
| TS §13.2; PS §9.3 | The storage check's second try, up to 4 s on a slow disk (Entry 12) |
| TS §13.3, §13.4; PS §11 | `LastExportDialog`, `SignInDialog`, `UpgradePrompt`, `AuthCallbackPage`, `AccountPage`; once per watermarked export (D-314) |
| TS §22.1, §22.2; PS §16 | The magic link's 503 (D-254); checkout's 403 (D-300); `users.email` normalized (D-260); rotation's grace and the lock (D-258, D-297); sessions end 60 days after sign-in |
| TS §22.4, §23.1, §23.3 | The outbox row's `plan` and `sub` (D-267); a receipt without a session waits (D-268); a 429 keeps its row (D-307) |
| TS §22.5 | The trait as built, `EventOwner`, `Resumed`, `PlanChanged`, the subscription id, the refusal, the mapping read from real deliveries, the refund's cancel at the merchant (D-274 to D-278, D-299 to D-301) |
| TS §22.8, §22.9 | Variables (D-251, D-273); `whsec_`; the retry read in TE-9 |
| TS §24.1 | The SMTP relay and the merchant's hosts |
| TS §37, §39.3 | TE-8 and TE-9 as read; items 3, 18, 20 and 22 |
| PS §17 | The address passes through Brevo |
| BP §8.1, §8A.0 | `smtp_mailer.rs`; `effective()`; "As built"; V6 at 14 prompts, the total about 147 |

TS §5 was kept right as files appeared. `server/src/auth/magic_link.rs`'s first comment said "the magic-link flow itself arrives in V6": it now says where the flow is.

**Step 14.1.** `git grep -nP "\bV6\b(?!\.\d)" -- web/src crates server`: no marker of work to do is left in `web/src` or `server`. Two comments in frozen crates still speak of V6 in the future (below).

**Tier 4 (D1, once).** `pnpm build`, `pnpm check`, `pnpm fixtures`, `pnpm test`, `pnpm e2e`, the verifier's self-test, a keyed `pnpm build:e2e`, then the full `pnpm e2e:media`, detached:

| Suite | Count | D-244's floor |
|---|---|---|
| Rust | 812 / 0 / 4 | 657 / 0 / 4 |
| Vitest | 522 | 336 |
| `pnpm e2e` | 45 | 12 |
| Verifier self-test | 16 | 16 |
| Media | 106 passed, 1 skipped (`rej_h264_444`, as before), in 35.1 min; nothing flaky | 94 + 1 skip |

Rust reads 812, as P144 recorded: P148's 816 summed lines from outside `cargo test --workspace`. No bench: no render path changed. The full media run is V6's one (D-242).

**Found, not fixed.**
- `crates/offcut-api-types/src/errors.rs` says the seven codes are "defined now so V6 adds handlers", and `crates/offcut-entitlement/src/token.rs` says `encode` is used "by tests now, and by the signer of V6". Both are true of the past now. The crates change only as comments in P137, so they wait for V6.1, which opens its own branch.
- From Entry 18: the INR price shown to a buyer in India, and the portal's plan change, unread.

**Open, for the founder, in order (G 14).** Open item 17 before the merge. G 14.4, Render's variables, with `BILLING_WEBHOOK_SECRET` a real `whsec_` value. G 14.5, ready and merge. G 14.6, the webhook for the deployed site, the live journey, the saved exports for `verify_mp4.py`, the production read. G 14.8, the tag. The closing entry follows the merge.

## Entry 20 - 2026-11-04 - V6 closed: merged, deployed, tagged

**Merge.** The founder merged pull request 10 as `f948429`, with the same tree as `638eaf5`. Tagged `v6` on `f948429` and pushed by the agent, at the founder's request.

**The `main` run, 41827888094, in three attempts.**

| Attempt | What happened | Done |
|---|---|---|
| 1 | `ci` green. `e2e-media` failed one case on both tries, the flake of known issue 3 (above); the deploy jobs were skipped, so nothing went out | The founder agreed to a re-run of the failed job |
| 2 | `e2e-media` 26 of 26. `deploy-api` pushed the image and called the hook, but Render kept the old instance: the new one stopped at boot with `config error: MAIL_FROM`. Render's value was not a bare address, which `lettre::Address` requires. `deploy-web` waited 300 s for the new version and stopped, so Vercel was not touched | The founder set `MAIL_FROM` to `.env.deploy`'s bare address. `healthz` read `f948429` at 13:14 UTC |
| 3 | `deploy-web` re-run: green | V6 live |

So the order held: the old API and the old site stayed together while the new API could not start. G 14.4 now says why: the check is the address alone, with no display name, quotes or spaces.

**Step 14.7.** `healthz` through the site reads `f948429`. `/account`, `/auth/callback`, `/legal/privacy` answer 200; `/api/v1/me` without a session 401. The live bundle holds V6's words. Headers on `/`: CSP as TS §24.3, COOP `same-origin`, COEP `require-corp`, `nosniff`, `no-referrer`, Permissions-Policy, HSTS.

**G 14.6, the live journey (founder and agent, test mode).**

| Step | Who | Read |
|---|---|---|
| Webhook endpoint for `offcut-api` | Founder | Registered and enabled in test mode (read through Dodo's API); its secret in Render |
| Sign-in with a real mail | Founder | Signed in on the deployed site |
| Free export | Founder | `offcut-20261104-1859.mp4`: 720 x 1280, watermark correlation 0.78, -14.1 LUFS; `verify_mp4.py` all 11 pass |
| Test purchase, $15/month | Founder | A new subscription at Dodo with `metadata.user_id`; the live account page read Creator, so the deployed webhook verified Dodo's signature and the row changed |
| Creator export | Founder | `offcut-20261104-1916.mp4`: 1080 x 1920, correlation -0.02, -14.2 LUFS; all 11 pass, with the expected length 45,200 ms of the video stream (the audio's 45,205 ms gives one frame more, so it is not the length to pass) |
| Cancel | Founder, in Dodo | The page read "It ends on 4 December 2026, and then you are on the Free plan": Creator kept to the period end (TS §22.5) |
| Full refund | Agent, through Dodo's API | `succeeded`, not partial |
| Account data | Founder | `offcut-account-data.json` downloaded |

**Not reported before the tag, so named here:** the account page after the refund (expected Free), the founder's read of the production `subscriptions` row (expected `canceled`, period end at the refund), and the deletion of the test account. The refund path is proven in test mode on D1 (Entry 18) and in `webhook_idempotency.rs` and `billing_events.rs`.

**Found, not fixed.**
- `web/vercel.json`'s rule for every path but `assets/` and `ort/` sets `Cache-Control: no-cache`, and so overrides the API's `no-store` on `/api/v1/*` through the site. It is from V1. A browser still revalidates, but responses that carry tokens should be `no-store`. For V6.1.
- Known issue 3, above, for V6.1.
- From Entries 18 and 19: the INR price shown to a buyer in India; the portal's plan change; two stale comments in frozen crates.

**Open after V6.** Open item 10: Dodo's product review, on hold as the live site read as a waitlist; with V6 live, the site now has free accounts and exports. Items 16 and 17 above. Then V6.1 opens its own branch from `main` (BP §8A.0).

**Final counts.** Rust 812 / 0 / 4; Vitest 522; `pnpm e2e` 45; verifier self-test 16; media 106 passed, 1 skipped, on D1. Every box of v6implementation §20 is ticked, each with its evidence.

## Entry 21 - 2026-11-08 - Founder's decisions for spec v1.4: two paths, 45-minute sources, stock B-roll (outside a prompt)

A brainstorming session after the cloud release closed (`cloudchangelog.md` Entry 12). Nothing in the code changed. The specs are not yet revised: these decisions are the input to the v1.4 revision, which comes before V6.1's plan is written. Where a line below contradicts TS v1.3, PS v1.3 or BP v1.3, this entry wins until v1.4 copies it in.

**Founder's decisions.**

| # | Decision | Replaces |
|---|---|---|
| 1 | **Two separate entry points and pages:** "Make shorts from a long video" and "Edit my clip". The person chooses the path; a file's duration no longer chooses it. Both paths open the same editor | TS §43.1's model line ("a file longer than 90 seconds is a source") |
| 2 | **One flow for both paths:** the transcript and a base prompt describing everything the editor can do go to the LLM; the plan is validated; **our editor applies it automatically**. The person sees a finished short and can switch any item off or undo. There is no approve-each-step screen. On the long path the LLM first returns the parts to cut and stitch (the Director), then one edit plan per short, exactly as for a clip | As designed (TS §40, §45); the "apply, then review" behaviour is now explicit |
| 3 | **The build order stays as BP §8A.0 sets it** (V6.1 first). Quality is inspected and improved after all features work, against a Submagic benchmark | BP §8A.0 confirmed; the "core first" reorder offered in the session was declined |
| 4 | **B-roll from two sources:** the person's own clips (for example a diagram or a lecture clip on "black hole", which stock cannot give) and **Pixabay stock**. The person can place B-roll on a specific word by hand ("Add B-roll here"); the plan also places it. Own clips need a choose-the-part trim | TS §48's model line (stock rejected, 2026-11-02) and its "Placement by the user" row (chat only, from V6.9). V6.8 keeps the feature |
| 5 | **Long sources up to 45 minutes**, and **no copy of the whole file**: the dropped file is read in place for the session, its audio is extracted and transcribed, the Director picks the parts, and only those ranges (from the keyframe before each part) are copied into OPFS | TS §43.2's 20-minute and 16 GB limits; §43.4's whole-file copy |
| 6 | **A long video requires cloud transcription.** If Groq fails, the job pauses with "Continue"; it does not fall back to the device (about 50-60 minutes for 45 minutes of speech on D1, derived). Clips keep the device fallback (D-348) | TS §43.6's on-device windows, its time budget and the two-pass seam |
| 7 | **Limits by duration and resolution only;** no file-size cap on the long path | TS §43.2's 16 GB |
| 8 | **Submagic-style auto zoom and visual editing** (zooms on key words, emoji, highlights, cards, sound cues, transitions) so a short is visual and engaging. Quality bar: Submagic and the founder's caption references | Extends V6.2's `PunchIn` and V6.6 part C |
| 9 | **Tooscut is the main study source for the editing engine:** read how it solves each problem, then design and write Offcut's own version, customised to shorts. Same rule as Entry 0: no copying or line-by-line translation of code, shaders, structs or constants; each idea learned is named in the changelog (TS §54) | Entry 0's rule, confirmed with more weight |

**What the decisions imply (the agent's reading, for v1.4).**

- **V6.1 gets smaller:** no 16 GB copy, no quota eviction for sources, no device transcription of a source, no two-pass seam. Range copying moves to where the parts are known (V6.3, V6.4). A tab closed before the parts are copied resumes by dropping the same file again (`sourceHashPrefix`, TS §43.7).
- **45 minutes is about 30 cloud windows of 90 s.** Groq's free tier (7,200 audio-seconds per rolling hour, 2,000 requests per day, read in TE-C1) carries about 2-3 such videos an hour, shared with every clip: the Dev tier is needed before the alpha, not at launch (D-342).
- **The Director's request grows** to about 7,000 words of transcript: the 64 kB request bound (BP §8A.4) and the body limit of `POST /shorts` rise to about 160 kB.
- **Pixabay** (terms read 2026-10-29, to be re-read before building): a free key, 100 requests per 60 s, searches cached 24 h, "from Pixabay" shown wherever results appear, commercial use without credit, `cdn.pixabay.com` sends `Access-Control-Allow-Origin: *`. A server search route keeps the key off the client and holds the cache. Search terms are transcript words that leave the device: the "What leaves" table, P-rules and `privacy-network.spec.ts` must name them. The plan proposes a search term at a word; the person can swap the result.
- **Own-clip B-roll** adds a Terms line: the person is responsible for the rights to clips they add.
- **The founder's caption references** (`sh2clips/documents/`, outside the repo; read by the agent as frame sheets) show three families beyond today's word-by-word captions: (a) **typographic lockups**: words of mixed size and weight stacked into one block, one accent colour, a rotated side word, the camera pushing through the block as it grows; (b) **mixed-font pairs** placed beside the speaker's head ("THIS" bold sans with "Style" in an italic serif), which needs a serif italic in the font registry and the face position of V6.5; (c) **glass cards** with icons, numbers and a soft glow, with the background blurred behind them. V6.6 part C and parts A/E are where these are weighed.

**Benchmark (founder).** A few Submagic-made shorts downloaded from YouTube, kept outside git (the repository is public), as the bar to beat in the final quality pass.

**Open items added:** 18 to 22 in the table at the top. Open item 6 (a 10-minute cap if TE-18 fails) no longer applies: long sources are transcribed in the cloud (decision 6).

## Entry 22 - 2026-11-08 - Spec revision v1.4: the core documents rewritten for Entry 21 (outside a prompt)

The founder asked for every core document to carry Entry 21's decisions, carefully. `technicalspec.md`, `product.md` and `buildplan.md` are now v1.4. Each change is marked "v1.4" where it sits, and v1.3's wording is kept beside it where a reader needs the history. No code changed. Uncommitted on the working tree; V6.1's setup commits it with this entry.

**Where each decision landed.**

| Entry 21 decision | TS | PS | BP |
|---|---|---|---|
| 1 Two pages | §43.1 (`Entry`, `route_probe` rules 11-15, `/app/long`), DC-1 rewritten | Positioning, §3, J2, L0, §9.4 | §0, §8A.1 |
| 2 One flow; the editor applies the plan | §40 unchanged (it already worked so) | §12.8 "applies the plan automatically", L6 | §8A.0 goal |
| 3 Order kept; quality pass at the end | §47.2 "The bar" | §9.5 row, E-15 in §19 | Table row, §8A.0, new §8A.12 |
| 4 Own and Pixabay B-roll, placed by hand | §48 model line, §48.5, new §48.6, §49.1-§49.2, P-17, INV-32, TE-26, §24.1 host 4, §24.3 CSP, §22.1 route, §22.8 key | §9.2, §9.5, §12.7, §12.8, §12.12, §17, §20.1, §21 item 14 | §8A.8, §8A.9, appendices |
| 5 45 minutes, read in place, parts copied | §43 rewritten, new §44.8 (`remux_range`, `SourceMedia`), §45.2, DC-15, §46, §50.3, §52, §53.3 | §9.1, §9.4, §12.9, §20.2A | §8A.1, §8A.3, §8A.4, §8A.5 |
| 6 Long path cloud only, pause on failure | §43.6, §43.7, §43.8, §16, §22.1 window budget, P-1, P-5, INV-3, INV-28, TE-18 | Principle 4, §9.2, §12.6, §12.7, L1, L3, L4, §17, §18 | §1.1, §8A.1 |
| 7 Duration and resolution limits only | §43.2, DC-3 | §9.4 | §8A.1 |
| 8 Auto zoom, Submagic-class look | §40.2, §47 table, new §47.2 | §9.5, §12.8 | §8A.2, §8A.6 part C |
| 9 Tooscut as main study source | §53.4 | §18 licences row | §0 paragraph |

**The agent's design choices, for the founder to confirm or change.** Entry 21 gave the decisions; these are the ways the specs now carry them, chosen by the agent:

1. **Pages.** `/app` stays the clip page, exactly as today (so every protected suite sees the page it knows), with one link to the new `/app/long`. A file of 90 s or less dropped on the long page opens as a clip. The clip page's "too long" panel gains a "Make shorts from it" button, with its words unchanged (DC-1).
2. **Clip-page file size** follows `high_res` instead of `long_video`: 16 GB with 4K input on, 500 MB off (DC-3).
3. **The sound is still written to OPFS** as `pcm16.f32` (about 173 MB for 45 minutes), so transcription, the Director and preparing shorts resume after a reload without the file. Only the audio step, the coarse vision pass and the part copies need the file again.
4. **Part copies** are one small MP4 per part, remuxed without re-encoding, with a 1 s handle each side (`PART_COPY_HANDLE`, assumption), built in V6.3 and called in V6.4 (TS §44.8). Adding a part later through chat asks for the original file.
5. **Cloud windows:** 90 s with 5 s overlap (32 windows for 45 minutes), 2 in flight, 2 retries, then a pause for one of four reasons (unavailable, busy, signed out, offline).
6. **Server limits for windows** (the founder sets the values): a window header on `POST /transcribe`, `LONG_AUDIO_PER_USER_DAY = 3 h` of audio per user per day instead of the 20 clips, 200 an hour per IP, and the global budget read from `TRANSCRIBE_BUDGET_HOUR_S` and `TRANSCRIBE_BUDGET_DAY_S` so the Dev tier raises it without code.
7. **`POST /shorts` takes 160 kB** (DC-15): a 45-minute request is about 115 kB.
8. **The coarse vision pass runs before transcription,** while the file is surely in hand.
9. **Auto zoom** is `PunchIn`'s new `motion: Snap | Push`, scales 1.12 and 1.25, one per sentence at most, never over a jump-cut zoom (all assumptions, E-11).
10. **Stock search** is `POST /api/v1/broll/search`, signed-in only, 20 a minute per person and 90 a minute for the server (under Pixabay's 100), cached 24 h by the query's hash so the words are never stored; the browser downloads the clip from `cdn.pixabay.com` through a new `net/stock-fetch.ts`. The plan writes a search term; `resolve-stock.ts` turns it into a local asset before validation, so the renderer only sees local files.
11. **The stock switch is on by default while the AI plan is on.** This is the founder's call at V6.8.
12. **Placing B-roll by hand** goes through `plan_ops` (`PlaceAsset`, `RemoveAsset`), built in V6.8 instead of V6.9, so it is one undo step and passes the same validator. The part picker (a window dragged along a clip's frames) is written into principle 3 as the one direct control besides the crop offset.
13. **The caption references** become three named candidates for V6.6 part C (typographic lockup, mixed-font pair beside the head, glass cards with a blurred background), each a cost check by name, not a commitment.
14. **The quality pass** is a step of its own after V6.11 (BP §8A.12, PS E-15): tuning inside the existing contracts; a new kind is a stop-and-ask.
15. **Estimates:** V6.1 12 prompts (was 14), V6.3 16 (was 14), V6.8 19 (was 16); about 150 in all with V6.

**Older errors fixed while editing.**
- PS principle 4 still said "Video and audio never leave", which has been untrue for clips since the cloud release. It now says what leaves.
- Two references in TS pointed at a "§54" that does not exist; they now point at §53.4, where the Tooscut table is.

**Checked.** Every "20 minutes", "16 GB", "64 kB", "copy" and "stock" in the three files was read after the edits; the remaining ones are history marked v1.3, the clip path's own import, or unrelated (CI runner times). Two table rows that an edit had joined were split again.

**Open items.** 18 is done, pending the founder's read of the choices above. New: 23 (the agent's choices 1-15 above, founder to confirm). Open items 19-22 stand.

## Entry 23 - 2026-11-08 - The founder confirms v1.4; V6.1's implementation plan (outside a prompt)

**Founder's confirmation.** On 2026-11-08 the founder answered "yes i confirm" to the agent's fifteen design choices of Entry 22. Open item 23 is closed; open item 18 is done.

**V6.1's plan written:** `docs/v6.1/v6.1implementation.md` (decisions D-383 to D-432; declared changes DC-1 to DC-4 only; prompts P171 to P182 after a Setup), modelled on `cloudimplementation.md` and V6's plan, after reading the code at `afb82bf`. TS §5 has its row. The build guide, the prompts file and `docs/v6.1/experiments.md` come next. This changelog stays V6.1's (BP §8A.0 step 1; D-384).

**What reading the code changed against the specs** (each copied back in P182, plan §18):

- **IndexedDB is at version 1 today,** not the "version 3" TS §43.4 assumed. V6.1 adds a structure version 2 opened only by the long path, keeps the records' `schemaVersion` at 1, and adds a `VersionError` rule so a later rollback can still open the database (D-414, plan §12.4).
- **The demuxer opens only an OPFS sync handle** (a web-sys type), so reading a `File` in place needs a new additive entry point in `offcut-wasm-core` with a structural reader (D-399).
- **The clip path decodes a whole track into one buffer** (about 520 MB for 45 minutes at 48 kHz), so the long path resamples in a stream: a new `StreamResampler` beside `resample_mono`, proved equal to it sample for sample (D-401).
- **HEVC is recognised by its four letters only;** `hvcC` parsing, the codec string and HDR from `colr` are new (D-396, D-397).
- **Feature flags all default to on** (cloud release, D-365). A V6.1 flag that did would ship at the merge, so flags get defaults and the V6.x flags default to off, turned on by `OFFCUT_FEATURES_ON` (D-393), with a second e2e build for the flag suites (D-394).
- **`POST /transcribe`'s IP limit is a route layer,** which cannot see a header, so the window header's rate groups move the IP check into the handler (D-411).
- **A clip import wipes `clips/` and every clip store** until V7's eviction, so sources live apart, and V6.1 keeps one source (D-415). `quota.ts` (BP §8A.1) moves to V7.
- **D-382's silence guard judges a word against the clip's own level;** a long source's quiet windows need the source's level, measured during the audio step (D-404, D-407).

**Open for the founder:** F-1 (45 minutes of the looped reference clip to Groq for TE-18), F-2 (a phone HDR clip for TE-21, or HDR stays rejected), F-3 (`high_res` on in production after V6.1), F-4 (`long_video` off in production until V6.4's alpha), F-5 (the copy of plan §10.5).

## Entry 24 - 2026-11-08 - The founder's answers; V6.1's guide, prompts and overnight run (outside a prompt)

**The founder's answers,** 2026-11-08, word for word: "yes to all". Read as (plan §15):

| # | Answer |
|---|---|
| F-1 | Yes: the looped reference clip's audio may go to Groq in P171 and P181 (about 5,400 s in all) |
| F-2 | Yes, the founder has a phone HDR clip. It is used locally only if placed at `sh2clips/testclips_private/hdr/` before the run, never committed or sent. **V6.1 ships with `hdr_ok = false` whatever TE-21 reads**: an unattended night does not widen what is accepted (the agent's reading, recorded in the plan) |
| F-3 | Yes: `high_res` on in production after V6.1 if TE-17's 4K memory read passes. It is the founder's morning step (a Vercel variable, then a re-run of `deploy-web`), not the night's |
| F-4 | Yes: `long_video` stays off in production until V6.4's alpha |
| F-5 | Yes: the copy of plan §10.5 as drafted |

**The founder's instruction for the build:** one unattended overnight session; "when I wake up tomorrow the feature must be implemented and merged, strictly without breaking, and do everything carefully". The agent is authorised to merge and tag under `overnight-run.md` §3 and §7, and never to change a production setting.

**Written:** `docs/v6.1/v6.1buildguide.md` (phases 0-12, the frozen and additive checks, tiers, 22 "bites"), `docs/v6.1/coding-promptsv6.1.md` (the SAB, Setup and Prompts 171-182, each with a "think first" rule and an overnight path for every stop), `docs/v6.1/overnight-run.md` (the master prompt: the founder's pre-sleep checklist, authorisations, safe paths, merge gates with a 06:30 cut-off, live checks, rollback, morning steps, report). Modelled on the cloud release's guide and prompts and the design release's overnight run. TS §5 has their rows; `check-file-tree.mjs` passes.

**Found while writing them** (each now in the guide):

- **An unattended session cannot sign in to a local API:** the magic link goes by the real relay (STARTTLS only) to an inbox nobody reads at night. P181 drives the real product build signed in through the e2e fake API, and real Groq calls go through a scratch proxy that calls Groq as `groq.rs` does (G Phase 11).
- **`.env.deploy` must never be sourced whole:** it holds the real mail relay and billing settings beside the Groq key. Scripts read `TRANSCRIBE_API_KEY` alone.
- **`fixtures/.gen-local/` and `web/dist-flags/` are not git-ignored at `afb82bf`;** P171 and P173 add them.
- **pnpm runs scripts through `cmd.exe` on Windows,** so `build:e2e:flags` wraps its environment prefix in `sh -c`, as `dev` does.
- **CI builds the keyed `web/dist` in the `ci` job and hands it to `e2e-media`;** the flags build follows the same path (plan §12.1, as read).
- **Production's flags come from the Vercel project's variables** (`vercel pull` in `deploy-web`), so turning a flag on is the founder's dashboard step plus a re-run of `deploy-web`, never the Redeploy button.

## Entry 25 - 2026-11-08 - V6.1 Setup: the branch, the documents, the floor

Built in one unattended session under `docs/v6.1/overnight-run.md`, started at 04:46 IST.

**Branch.** `v6.1-build` from `origin/main` at `2568a6e` (checked). Cherry-picked `429d8aa` and `afb82bf` (the cloud release's closing entries) as `1f71eca` and `277b0d6`; they change `docs/cloud_transcription/cloudchangelog.md` only. This commit holds the v1.4 specs (`technicalspec.md`, `product.md`, `buildplan.md`), Entries 21-25 of this changelog, and `docs/v6.1/` (plan, guide, prompts, overnight run, and the new `experiments.md` with one heading per experiment, "Not read yet"). TS §5 gains `experiments.md`'s row; `check-file-tree.mjs`: ok, 551 files.

**The founder's answers** are Entry 24's ("yes to all", F-1 to F-5). **Before sleeping, the founder also answered the run's own questions** (2026-11-08, 04:50 IST), recorded in `overnight-run.md` §12 (A-1 to A-7), which wins over its §7.2:

- A-1: the run began too late for §7.2's "merge before 06:30". The merge may start **whenever every §7.2 gate is green, never after 22:00 IST**; the work continues after the founder wakes.
- A-2: no HDR clip. TE-21 is read on `rej_hdr_hlg.mov` only; `hdr_ok` stays `false`.
- A-3: the real Groq calls go through production's key as planned (about 5,400 s, at most 3,000 s an hour), stopping if Groq's remaining hourly or daily audio falls under 25%.
- A-4: if free disk falls under 15 GB, the whole `target/` may be deleted (66 GB free at the start).
- A-5: narrow local allow rules for the pushes, the pull request and the merge, in a git-excluded `.claude/settings.local.json`; pushes to `main`, force-pushes and deletes are denied. Removed at the close.
- A-6: phone notifications at key moments only. A-7: building goes on while the founder uses the laptop; a media case failing only under load is re-run once.

**The floor (V6.1's baseline, BP §8A.0 step 1),** read on D1:

| Suite | Read | D-386 floor |
|---|---|---|
| Rust (`cargo test --workspace`) | 863 / 0 / 4 | 863 / 0 / 4 |
| Vitest | 568 (24 files) | 568 |
| `pnpm e2e --workers=4` (after `pnpm build`, `check-hosts.mjs` ok) | 57 passed | 57 |
| Media | not run (D-385); floor 112 passed, 1 skipped (CT CL Entry 11) | |

**Checks.** `git diff --name-only origin/main` lists docs only. No product code changed.

**Open:** nothing new. **Found, not fixed:** none.

## Entry 26 - 2026-11-08 - P171: the probes

No product code. Every reading, with its numbers, is in `docs/v6.1/experiments.md`; the scripts stayed in the scratchpad.

- **`.gitignore`** gains `fixtures/.gen-local/` before anything was written there (G item 18); `git check-ignore` confirms it.
- **Local long files** (D1 only, never staged): `long_45min_4k_hevc.mov` (1.48 GB), `long_45min_1080p_h264.mp4` (5.61 GB, past 4 GiB on purpose), `hevc_main10_2s.mov`, from the reference clip's sound looped with 5 s of room tone, made in about 2 minutes by encoding a 10 s video segment and copying it in a loop.
- **TE-17, first read:** `FileReaderSync` in a worker reads 1 MiB blocks in 3-7 ms p50 (warm cache), reads `moov` (3 MB) in under 9 ms, and **reads blocks past 4 GiB correctly** (7 of 7 hashes equal to Node's); page memory stays near 1 MB. D-400 stands.
- **HEVC:** D1's Chrome decodes Main and Main 10 through hardware only (strings from real `hvcC`: `hvc1.1.6.L150.90`, `hvc1.2.4.L120.90`, `hvc1.1.6.L93.90`). P171's "no HEVC" stop does not fire.
- **Long AAC:** 45 minutes through one `AudioDecoder` in 5.3 s, 48 kHz on every output, 9.7 MB peak. HE-AAC could not be made on D1 (no encoder); it stays a unit-test case and TE-12's.
- **Audio track:** `demux.rs` reads the first audio track in file order already; D-402 needs no new demuxer code.
- **TE-18, first read (F-1), the decision of the night so far.** 32 windows of the looped reference sound to Groq: 2.8 s p50, 5.9 s p90 per window, about 80 s for all 32 two at a time. Merged as `mergeWindows` merges, **3 of 31 seams lost or doubled a word** (1 lost, 2 doubled), and 25 of 32 windows had words at their edges: TE-18's seam line fails and P171's stop condition fires. **ON §6's safe path: `CLOUD_OVERLAP_S = 10`** (step 80 s, 34 windows for 45 minutes, `MAX_SOURCE_WINDOWS = 35`), written as "As read" under D-405 and D-411. P181 re-reads it through the product.
- **Found: Groq's free tier limits requests per minute.** A 429 (`retry-after: 3`) came after 27 requests in about 66 s while 4,872 of the hour's 7,200 audio seconds were left. D-409's two in flight (confirmed by the founder, Entry 23) is kept; open item 24 asks the founder to choose the Dev tier or one in flight before `long_video` goes on.

**As read in the plan:** D-398 (HEVC answers), D-402 (the audio track), D-405 (10 s overlap), D-409 (latency and the per-minute limit), D-411 (`MAX_SOURCE_WINDOWS = 35`).

**Real Groq calls (F-1, `overnight-run.md` §12 A-3).** The looped reference clip's sound only, through production's `TRANSCRIBE_API_KEY` read alone by the script's process.

| When (IST) | Windows | Audio sent | Statuses |
|---|---|---|---|
| 05:06:36-05:07:46 | 0-28 (two in flight) | 28 × 90 s answered | 28 × 200, 1 × 429 (window 27) |
| 05:08:43-05:08:56 | 27, 29, 30, 31 | 3 × 90 s + 65 s | 4 × 200 |
| **Total** | 32 windows, 33 requests | **2,855 s** (under the hour's 3,000 s) | Groq's hour: 4,597 of 7,200 s left at the end |

**Counts:** unchanged from Entry 25 (no code): Rust 863 / 0 / 4, Vitest 568, `pnpm e2e` 57.

**Open:** item 24 (new). **Found, not fixed:** the per-minute limit above (a founder decision, not a code fix).

## Entry 27 - 2026-11-08 - P172: types, HEVC, colr and `route_probe`

**Built.**

- `offcut-types/src/source.rs` (new): `Entry`, `RouteFlags`, `VideoDetail`, `SourceInfo` (with `is_hdr`, `avg_fps`, `video_codec`, `audio_track` 0, `audio_tracks`), `Route`, `SourceReject` (`REJECT_HDR`), `RouteReject` (untagged: a V5 `RejectReason` or a `SourceReject`), `SourceErrorCode` (`E_SOURCE_READ`, `E_SOURCE_AUDIO`), the constants `MAX_SOURCE_DURATION` (45 min), `MAX_INPUT_LONG_SIDE` (3840), `MAX_FILE_SIZE_HIGH_RES` (16 GB, DC-3), `CLOUD_WINDOW_S` 90, `CLOUD_OVERLAP_S` **10** (P171), `MAX_SOURCE_WINDOWS` 35, and `window_count()`. `lib.rs` adds the module and, in its generator, the eight types after every V5 type, `SOURCE_LIMITS`, `SOURCE_REJECT_REASONS` and `SOURCE_ERROR_CODES`.
- `offcut-mp4`: `hevc.rs` (`parse_hvcc`, `codec_string` per Annex E), `colr.rs` (`parse_colr`, `is_hdr`), `route.rs` (`route_probe`, §6.2's 17 rules; rule 1 returns `validate_probe`'s answer as it is; a private `clip_info` builds the `ClipInfo` as `validate_probe` does), `demux.rs` gains `VideoEntryBoxes` and `Demuxer::video_entry_boxes()` (a second walk to the track `open` chose, for `hvcC` and `colr`), `probe.rs` gains `video_detail()`, `validate.rs` gains the one line `pub use crate::route::route_probe;`.
- `pnpm gen:types`: `web/src/gen/domain.ts` gains 83 lines, removes none.

**DECLARED CHANGE / REOPENED CONTRACT.** Additions only in `offcut-types` and `offcut-mp4` (D-387): G 0.4's additive check prints nothing for `validate.rs`, `media.rs`, `error.rs`, `limits.rs`, `demux.rs`, `probe.rs`, both `lib.rs` and `demux_fixtures.rs`. No existing function body changed; `validate_probe`, `probe`, `read_stsd` and `video_decoder_description` are as they were.

**What differs from the plan (As read under D-395 and D-397).** V5's tests pin `RejectReason` and `ErrorCode` (exhaustive matches and the TypeScript unions' exact counts), `limits.rs` (21 constants) and every `VideoTrackInfo` literal; adding a member, a constant or a field would have meant editing those cases, which ON §3 forbids. So the codes and the rules are the plan's, but `REJECT_HDR` and the two `E_SOURCE_*` codes live in their own enums, the constants in `source.rs`, and `is_hdr` with the HEVC string in a `VideoDetail` passed beside the `ProbeInfo`. `colr.rs` reads `nclc` as well as `nclx` (ffmpeg writes `nclc` in a `.mov`). `SourceId` is left to TypeScript (P176). The copy check learns the two new code arrays in P179, with their copy.

**Tests (all new; no existing case touched).** `source.rs` 6 (window counts for a 10 s overlap, the limits' order, the codes' serialization); `lib.rs` 1 (the generated long-path types and arrays); `hevc.rs` 4 (three real `hvcC` headers give `hvc1.1.6.L93.90`, `hvc1.1.6.L150.90`, `hvc1.2.4.L120.90`; Annex E's space, tier and inner zero bytes; malformed); `colr.rs` 5; `demux_fixtures.rs` 1 (`rej_hevc.mov`'s `hvcC` and codec string, H.264 clips have none); `route_probe.rs` 10: with both flags off `route_probe` equals `validate_probe` on all 24 fixtures the demuxer opens, with decode yes and no, from both pages; with `high_res` on every fixture V5 accepts is the same `ClipInfo` (and opens as that clip from the long page); `rej_4k.mp4` and `rej_hevc.mov` accepted with `high_res`; every rule alone from the long page; resolution, rotation and HEVC with `high_res`; the clip page keeps V5's duration and track rules; the earlier of two failing rules wins; the long page with its flag off is the clip page; a two-track source reads track 0; 45:00 is long, 45:00.001 is refused.

**Counts.** Rust 890 / 0 / 4 (863 + 27; `cargo test -p offcut-types -p offcut-mp4`, the workspace is P182's), Vitest 568, `pnpm e2e` 57 (no web code changed). `pnpm check`: fmt, clippy, deny, eslint, both `tsc`, file tree (556 files: rows for `colr.rs`, `route.rs`, `route_probe.rs`), gen clean, copy codes: ok. Frozen check empty.

**Carried to P173.** The fixtures' manifest entries must name the code V5 gives (`rej_hdr_hlg.mov` is HEVC, so `REJECT_HEVC` for `validate_probe`), since `probe_rejections.rs` and `demux_fixtures.rs` read the manifest with `deny_unknown_fields` and expect `validate_probe`'s reason; `REJECT_HDR` is asserted in `route_probe.rs`. The HLG fixture's `colr` case joins `demux_fixtures.rs` then.

**Open:** nothing new. **Found, not fixed:** none.

## Entry 28 - 2026-11-08 - P173: the stream resampler, the WASM entries, the flags, the fixtures

**Built.**

- **`StreamResampler`** (`offcut-dsp/src/resample.rs`, below `resample_mono`, additive): `push` and `finish`, the same `rubato` `Fft` calls `process_all` makes, the start-up delay dropped from the head, the same zero pump and length rule. **It equals `resample_mono` bit for bit** (no fallback needed): `stream_resample.rs` (new, 4 cases) at 48 and 44.1 kHz in random pieces of 1 sample to 10 s, around every chunk edge, equal rates, and a rate change mid-stream. The long case runs 2 minutes of noise, not the plan's 7, which took 128 s in a debug build.
- **A demuxer for 45 minutes.** Found: `Demuxer::open` expands no track over 20,000 samples (V2 D-60), so it cannot read a long source's audio (45 minutes of AAC is about 126,600 samples). Added, without touching `open`: `sample_table::resolve_up_to` and `Demuxer::open_source` (the same steps with `MAX_SOURCE_SAMPLES = 400,000`, about 16 MB a track at most), and `Demuxer::audio_sample_meta` (a sample's times without its bytes, for the region search). A unit test proves `open_source` reads what `open` reads; `demux_fixtures.rs` reads `long_12min.mp4` (33,750 audio, 21,600 video samples) to its last sample, where `open` cannot.
- **WASM** (`offcut-wasm-core/src/source_api.rs`, new): `JsByteReader` (any object with `size()` and `readAt()`), `open_demuxer_reader` with a `SourceDemuxerHandle` (probe, `video_detail`, `hevc_description`, the descriptions, `audio_sample_time_us`, `read_audio_sample`), `route_probe` (one pure export over a `ProbeInfo` and a `VideoDetail`), the `StreamResampler` class. A file that cannot be read is `E_SOURCE_READ`, a broken sample `E_SOURCE_AUDIO`. `media_api.rs` gains, additively, `video_detail` and `hevc_description` on `DemuxerHandle` (the clip page with `high_res`). `load-core.ts` gains `openDemuxerReader`, `routeProbe`, `newStreamResampler` and the two `CoreDemuxer` methods. `pnpm build:wasm` builds.
- **Flags (D-393, REOPENED CONTRACT `features.ts` and `vite.config.ts`).** `FeatureFlags` has `long_video` and `high_res`; `DEFAULTS = { cloud_transcribe: true, long_video: false, high_res: false }`; `featureFlags(on, off)` moved from `vite.config.ts` into `features.ts` (exported, so a test reaches it), reading `OFFCUT_FEATURES_ON` beside `OFFCUT_FEATURES_OFF`; an unknown name or a name in both stops the build. `features.test.ts` (new, 3 cases). `cloud_transcribe`'s behaviour is unchanged.
- **Two builds (D-394).** `package.json`: `build:e2e:flags` (through `sh -c`, G 3.3; run on D1) and `e2e:media:flags` (its project comes in P180). `web/dist-flags/` is git-ignored. **Read:** `pnpm build`'s bundle holds `cloud_transcribe: true`, `long_video: false`, `high_res: false`; `dist-flags/` holds both V6.x flags `true` (the define over the `false` defaults).
- **Fixtures (D-430).** `gen_fixtures.sh` step 5c makes `long_12min.mp4`, `rej_too_long_46min.mp4`, `rej_8k.mp4`, `rej_hdr_hlg.mov`; **they are listed in `fixtures/manifest-long.json`, not `manifest.json`**, because V5's `probe_rejections.rs` requires every `rej_*` entry of `manifest.json` to be one of its twelve cases. The script checks, probes and stamps both manifests (the stamp gains `long_manifest_sha256`). HLG needed `setparams` and `-movflags +write_colr`; the file carries `colr` `nclc` 9/18/9 (P172's `nclc` reading confirmed). **Time:** the four add about 30 s to `pnpm fixtures` on D1 (23.1 s, 6.1 s, 0.4 s, 0.6 s; the 46-minute room tone at 16 kHz cut it from 27 s); the whole run is 114 s on D1. Under G item 19's 60 s.
- `scripts/gen-long-fixture.sh` (new, D1 only): P171's 45-minute files, refusing to run unless `fixtures/.gen-local/` is git-ignored.

**New tests.** `stream_resample.rs` 4; `demux.rs` 1 (`open_source` equals `open`); `route_probe.rs` 3 (the four fixtures route as `manifest-long.json` says from all three entries; the HLG file is HDR Main 10; `long_12min.mp4` is a 12-minute source of 9 windows); `demux_fixtures.rs` 1 (`long_12min.mp4` to its end, the HLG `colr`); `source_api.rs` 1; `features.test.ts` 3. No existing case changed; `manifest.json` is unchanged.

**Checks.** `pnpm check` ok (560 files; TS §5 rows for `source_api.rs`, `stream_resample.rs`, `features.test.ts`, `manifest-long.json`, `gen-long-fixture.sh`). Frozen check empty; additive check: no line removed from `validate.rs`, `resample.rs`, `media_api.rs`, `demux.rs`, `sample_table.rs`, `load-core.ts`, `demux_fixtures.rs` or `manifest.json`.

**Counts.** Rust 900 / 0 / 4 (890 + 10; per crate), Vitest 571 (568 + 3), `pnpm e2e` 57 (not re-run: no page changed). Media: none run.

**As read in the plan:** D-399, D-401, D-430. **Open:** nothing new. **Found, not fixed:** none.

## Entry 29 - 2026-11-08 - P174: the media worker reads a file in place

**Built.**

- `workers/media/file-random-access.ts` (D-400): a `ByteReader` over the dropped `File` through `FileReaderSync`, 1 MiB aligned blocks, the last 8 kept; any failed read is `SourceReadError`. Test: 4 cases (within and across blocks, the cache and its eviction, `NotReadableError`, a file shorter than it said).
- `workers/media/audio-region.ts` (§7.2): `decodeAudioRegion` (from the sample at or before `from − 100 ms` found by a binary search over `audio_sample_time_us`, `decodeTrack`'s batching and queue, mono, audio before 0 and from `to` on dropped, chunks of at most 10 s, the decoded rate reported with each) and `writeWholePcm16` (the file cut to the resume point, the part already written re-read into the level histogram, a `StreamResampler` per rate, silence for any gap and the tail, a flush and a checkpoint every 10 s, `round(duration × 16)` samples), `LevelHistogram` (0.1 dB bins; its 95th percentile within 0.1 dB of `cloud-audio.ts`'s sort). Test: 7 cases with a fake demuxer, decoder and resampler (placement, audio before 0 dropped, **a resume from 30 s equals the run from 0 bit for bit**, a rate change rebuilds the resampler and keeps the length, the level, cancel, a decoder failure).
- `media.worker.ts` (additive; the handlers line is the one changed): `routeFile` (reads in place, asks `isConfigSupported` with the `avcC` or the `hvcC`, `routeProbe`), `importAndRoute` (the 16 GB rule with `high_res`, the V5 copy, `routeProbe` from the clip page), `extractSourcePcm16` (into `sources/<id>/pcm16.f32` through a sync handle). `protocol.ts` gains the three methods, `SourceId`, `Routed`, `SourceFailed` (additive). `opfs.ts` gains `sourcesRoot`, `sourceDir`, `sourcePcm16` (additive; §9.2's paths, wanted here first).

**DECLARED CHANGE / REOPENED CONTRACT:** `protocol.ts`' additive members (D-387); `media.worker.ts`' handlers line. `import.ts`, `audio-decode.ts`, `rpc.ts`, `rpc-worker.ts`, `pool.ts` and the render worker are untouched (frozen check empty).

**What differs from the plan (As read under D-403 and D-427).**

- **E_SOURCE_* as values.** `rpc.ts` (frozen) turns any code that is not a V5 `ErrorCode` into `E_INTERNAL`, so the source handlers return `{ failed: "E_SOURCE_READ" | "E_SOURCE_AUDIO", detail }`. Storage failures still throw their V5 codes.
- **The 16 kHz grid.** The scratch smoke found a resumed run 0.31 apart from the run from 0: a resampler started on a packet boundary is shifted by a fraction of a 16 kHz sample. A resampler now starts on an input sample on the 16 kHz grid (at most 2 samples skipped at 48 kHz, 440 at 44.1 kHz, inside the pre-roll).
- **HEVC on the clip page.** Found: the render worker (frozen) configures its decoder with the `avcC` only, so an HEVC clip could be accepted and then fail in preview and export. `importAndRoute` refuses HEVC with `REJECT_HEVC` even with `high_res` (ON §6's "H.264 up to 4K only"); `high_res` on `/app` means 4K H.264 and 16 GB. The long page reads HEVC sources' sound. Open item 25.
- `decodeAudioRegion` takes the opened demuxer, not a reader; the handler opens it. The checkpoint reaches the main thread as throttled progress (`stage: "import"`, milliseconds of flushed sound); a lagging checkpoint costs only re-decoding.

**The scratch smoke (G Phase 4, Tier 0)** on `long_12min.mp4` through the real media worker under the Vite dev server (a `zz-` page, deleted): `routeFile` 412 ms (in place); `extractSourcePcm16` 5.4 s for 12 minutes; `pcm16.f32` **11,520,000 samples, exactly `round(720,000 × 16)`**; `levelP95` −20.3 dBFS; **peak page memory 14 MB** (no buffer of the file or the track). **Correlation with ffmpeg's own decode:** the first second is near-digital silence (RMS 1.1e-5), where the figure means nothing (0.93); a speech second gives 0.986 against ffmpeg's resampler and 0.986 against soxr, at lag 0. The rest is the decoder (Chrome's against ffmpeg's) and the placement `decodeTrack` shares, which V5's sync proofs cover; the guide's 0.99 is not met on this measure, recorded. **Resume in the browser:** from 300 s, the same length; equal to 1e-6 for 3.3 s, then within 0.0027 (−51 dBFS) where the FFT resampler's blocks fall elsewhere.

**Counts.** Rust 900 / 0 / 4 (unchanged), Vitest 582 (571 + 11), `pnpm e2e` 57 (not re-run). Checks: `tsc` (both), eslint, file tree (564; rows for the four new files). Media: none run.

**Open:** item 25 (new). **Found, not fixed:** the 0.986 above, recorded rather than chased: the long path's sound is the clip path's sound by construction.

## Entry 30 - 2026-11-08 - P175: the server, windows, budget, events, no-store

**Built.**

- **The window header (D-411, REOPENED CONTRACT CT D-363).** `transcribe/route.rs`: `window_of()` parses `X-Offcut-Window: <index>/<total>` (`0 ≤ index < total ≤ MAX_SOURCE_WINDOWS = 35`, digits only); absent is a clip, as before. The handler now runs: the header (malformed is 400, logged `bad_window`); **the address's bucket** (`TranscribeWindowIp` or `TranscribeIp`, from `client_ip` as the layer had it); **the bearer**, extracted by hand after the address's bucket so an unauthenticated request still takes its tokens; the person's bucket (`TranscribeWindowUser` or `TranscribeUser`); then steps 3-8 unchanged. `router.rs` drops the route's `ip(…)` layer and keeps `body(MAX_AUDIO_BYTES)`. The log line gains `window: true/false`, never the index. `rate_limit.rs`: `TranscribeWindowUser` 120 a day, `TranscribeWindowIp` 200 an hour (members at the end).
- **The budget from configuration (D-412).** `config::transcribe_budget()` reads `TRANSCRIBE_BUDGET_HOUR_S` and `TRANSCRIBE_BUDGET_DAY_S` (optional; unset or empty keeps 7,000 and 27,000; anything else but a whole number above 0 stops the boot with its name); `main.rs` builds the `Budget` from them. `budget.rs` is unchanged. `render.yaml` lists both, `sync: false`, unset.
- **No-store reaches the browser (D-429).** `web/vercel.json`'s no-cache rule is `/((?!assets/|ort/|api/).*)`. `check-headers.mjs` gains a static assertion that no no-cache rule covers `/api/v1/*` (it fails on the old rule; read) and passes against `vite preview`.
- **D-428's three analytics events: not built.** `analytics.rs`' tests pin the list (20 samples zipped with the docs, the union's count); adding events means editing them, which ON §3 forbids. `long_video` is off in production until V6.4, so nothing would be sent before then. Open item 26; As read under D-428.

**What differs from the plan (As read under D-411, D-412, D-428).** `Config` gains no field: four test files build it as a literal. The log check is a new binary, `server/tests/transcribe_window_log.rs` (TS §5 row): in `transcribe_route.rs` it raced the other tests over tracing's process-wide callsite state (it passed alone, failed in parallel), as `log_redaction.rs`' own note warns.

**Tests.** Every existing case of the server's test files passes **with no change to its code** (`git diff $BASE -- server/tests/` adds cases and one file only). New: `transcribe_route.rs` 4 (windows answered as clips; malformed headers 400 before bearer and body; the 121st window of a person in a day is 429 while their clips pass; an address's clip and window buckets are apart, 60 and 200); `transcribe_window_log.rs` 1; `route.rs` 1 (the header's grammar); `config.rs` 1; `rate_limit.rs` 1.

**Counts.** Rust 908 / 0 / 4 (900 + 8), Vitest 582, `pnpm e2e` 57 (not re-run; `check-headers` against `vite preview` ok). No migration; no new dependency.

**Open:** item 26 (new). **Found, not fixed:** none.

## Entry 31 - 2026-11-08 - P176: storage and state

**Built.**

- **`schema.ts`** (additive): `DB_STRUCTURE_VERSION = 2`; `SOURCE_STORES` (`sources`, `sourceTranscripts`, `masterTranscripts`), apart from `STORES`; `SourceId`, `SourceStage`, `SourceRecord`, `SourceWindowRecord`, `MasterTranscriptRecord`, `OffcutSourcesDb`. `DB_VERSION`, the records' `schemaVersion`, stays 1.
- **`db.ts` (REOPENED CONTRACT, D-414).** Upgrade 2 creates the three stores; the upgrade loop runs only up to the version being opened (a fresh clip-path open at 1 never makes them); `open()` falls back to the database's own version on `VersionError`; `openSourcesDb()` opens version 2, only for `sources-repo.ts`. `openDb()` and its callers are unchanged.
- **`opfs.ts`** (additive): `readPcm16Window(id, k)`, `[80k, 80k + 90)` seconds.
- **`sources-repo.ts`**: put, get, update, newest, windows by `[sourceId, index]`, the master transcript, `removeSource` (rows, windows, transcript, `sources/<id>/`) and `removeAll` (D-415's sweep; clips untouched); the newer-`schemaVersion` rule; failures as `OpfsError`.
- **`source-machine.ts`** (§9.4, D-417): 11 states, 20 events, 33 legal pairs, `transcribed` V6.1's end. **`source-store.ts`**: the status through the machine, the page's state (gate, pause, progress, transcript, resumable, a different file). **`blockers.ts`** (D-416, additive): `forImport()` answers `B_PIPELINE_BUSY` while a source job works; new `forSource()` answers it while a clip is busy or an export runs. Its both-ways test is P178's (`import-source.test.ts`, §11.3).

**Tests.** `sources-repo.test.ts` 5 (a mocked database, as `receipt-outbox.test.ts` does: the project has no fake IndexedDB and gains no dependency); `source-machine.test.ts` 37 (every legal pair through `transition` and the store, every other pair refused, one or more per state, the pairs that must never happen). **The upgrade itself, read in Chrome** (a `zz-` page under the Vite dev server, deleted): the clip path writes version-1 records, the long path upgrades to 2, every version-1 record is byte for byte the same, the three stores exist, and the clip path reads its records afterwards through the `VersionError` fallback.

**Tier 2 on the keyed `dist/`.** `pipeline-preview.spec.ts`' first case ("feed: …") and its storage case ("OPFS: …"): passed, with `media-setup` (2.3 min). No media spec reloads and restores a clip, so G Phase 6's "restore cases" are these two, which write through the changed `db.ts`.

**What differs from the plan (As read under D-414).** `SOURCE_STORES` apart from `STORES`; `openSourcesDb()` in place of `openDb({ structure: 2 })`; the bounded upgrade loop; the `VersionError` rule is needed within this build, not only for a rollback.

**Counts.** Rust 908 / 0 / 4, Vitest 624 (582 + 42), `pnpm e2e` 57 (not re-run). Media: 2 cases on `dist/` (above).

**Open:** nothing new. **Found, not fixed:** none.

## Entry 32 - 2026-11-08 - P177: transcription in cloud windows

**Push 1** (P176's commit `2cd61f2`): CI run `42447624770`, read at P182.

**Built.**

- **`cloud-audio.ts` (DECLARED, D-407):** `dropSilentWords(words, pcm16, refLevel?)`; absent, the window's own 95th percentile as before, so `fromCloud`'s call and every existing case are unchanged.
- **`asr.worker.ts`** (handlers line and two import lines changed): `cloudWindowWords` (the guard against the source's level, words relative to the window, unmerged) and `buildMasterTranscript` (`mergeWindows`, `postProcess`, `normalizeTranscript`; fewer than `MIN_WORDS` is `NoSpeech`). Neither touches the model session. `protocol.ts` gains both and `WindowWord` (additive).
- **`net/http.ts`** (additive): `HttpRequest.window` sends `X-Offcut-Window: <index>/<total>`. **`api-client.ts` (DECLARED, D-413):** `transcribeAudio(ogg, signal, window?)`.
- **`usecases/transcribe-source.ts`** (§8.1): `windowCount` (10 s overlap: 9 windows for 12 minutes, 34 for 45), the missing windows in order, two in flight, each kept in `sourceTranscripts` before it counts; D-410's paths: 429 pauses `busy` at once; an `E_AUTH_*` (the session gone after `authed()`'s refresh) pauses `signed_out`; 503 `transcribe_unavailable` asks `/healthz`, off pauses `unavailable`, on is retried; a timeout, offline or other failure is retried after 1 s and 4 s, then pauses `offline` or `busy`; a pause aborts the other request in flight and keeps what answered before; then the master transcript is built and kept. No device fallback. An encoder this browser cannot use ends `{ failed: "encoder" }` (not in D-410; the caller fails the job with `E_SOURCE_AUDIO`).

**Tests.** `transcribe-source.test.ts` 10 (counts and offsets; order and at most two in flight; resume sends only what is missing; 429; timeout twice then kept with waits 1 s and 4 s; offline and failures to their pauses; 503 with `/healthz` off and on; a lost session; a pause stopping the other lane; `NoSpeech`; the encoder). New cases in existing files (no existing case changed): `api-client.test.ts` 1 (**a clip's request has today's headers exactly; a window's adds only `X-Offcut-Window`**), `cloud-audio.test.ts` 3 (a quiet window's invented words kept against its own level, dropped against the source's; a real word kept).

**Counts.** Rust 908 / 0 / 4, Vitest 638 (624 + 14), `pnpm e2e` 57 (not re-run). Media: none.

**Open:** nothing new. **Found, not fixed:** none.

## Entry 33 - 2026-11-08 - P178: the job, import, run, resume, hand-over

**Built.**

- `platform/wake-lock.ts` (D-418): `holdAwake(onLost)` takes the screen lock, takes it again when the tab is shown, reports once a lock lost while visible that cannot be taken again; `showProgressInTitle` ("42% - Offcut", then the page's title back).
- `usecases/import-source.ts` (§9.3): `dropSource` (blockers, the hash prefix, `routeFile` in place; a refusal, a failure, or a short file handed to `/app` through `handOff`/`takeHandOff`, memory only, read once), the gate in order (signed in; cloud, waiting up to 45 s; room on the disk, `round(D × 16) × 4 B + 2 GB`; then `ready`, the line on what is sent and Start), `recheckGate`, `startSource` (the previous source swept, D-415; the record made at `extracting_audio`; the job run).
- `usecases/run-source-pipeline.ts` (§9.5): extraction with the file (checkpoints every 10 s of flushed sound, D-403; the level and `windowsTotal` kept), then the windows; pause, failure and the end let go of the wake lock and the title; `cancelSource` removes the source; the file stays in hand for Continue while the page lives.
- `usecases/continue-source.ts` (D-419): `restoreSource` (at start, `long_video` on only: the newest source offered as "Continue your video", or its transcript shown), `resumeSource` (transcription goes on at once, extraction waits for the file), `dropSameFile` (size and hash prefix both match, or "This is a different file"), `continueSource` (after a pause or a failure, back to the stage it left; a pause during extraction with the file gone asks for it).
- `start-app.ts` (step 9, D-419): `restoreSource()` only when `FEATURES.long_video`, so production never opens the source stores.
- `import-clip.ts` (D-427, D-421): with `high_res` on, `importAndRoute`; otherwise today's `importAndProbe`, unmoved in the `else` (`git diff -w` shows only `probed`'s type widened); `REJECT_HDR` is shown but not sent as `clip_rejected` (open item 26); with `long_video` on, `REJECT_DURATION` hands the `File` to the long page ("Make shorts from it").
- `clip-store.ts`: `reject` is a `RouteReject` (V5's codes and `REJECT_HDR`). `copy/long.ts` (new): §10.5's strings as approved, `REJECT_HDR`, `E_SOURCE_READ`, `E_SOURCE_AUDIO`; `messages.ts` gains `long` and spreads `REJECT_HDR` into `rejections` (`satisfies Record<RouteReject, …>`); 673 of 700 lines.
- `eslint.config.js`: §4.1's edges (`run-source-pipeline → transcribe-source`; `import-source`, `continue-source → run-source-pipeline`; `import-clip → import-source`) and `start-app → continue-source`.
- `crates/offcut-text/tests/sentences_long.rs` (new): 45 minutes of Script A's words looped with room tone: every word in one sentence, in order; none over 40 words; none across a loop's pause; a stretch written without stops split at its 400 ms pauses; every sentence of this corpus within TS §43.6's 25 s.

**What differs from the plan.** The job's cancel is its own controller in `run-source-pipeline.ts` with `pool.cancel("media")`, not a `"source"` scope of `cancel-job.ts`. The gate ends in `ready` with Start (§10.2's last line), so nothing is written before Start. `sentences_long.rs` builds its corpus from Script A, since the fake windows that write the 12-minute master transcript are P180's. TS §43.6 names `MAX_SENTENCE_MS = 25 s`; `offcut-text` splits by 40 words and may not change (D-387): the test holds 25 s over this corpus, and the rule itself is for the release that may reopen `sentences.rs` (Found, not fixed).

**Tests.** `import-source.test.ts` 6 (each gate in order, Start with the sweep, the hand-over once, a refusal and a failure, the blockers both ways); `run-source-pipeline.test.ts` 5 (stages in order with the checkpoint, the wake lock and the title; a resume without the file; no file asks for it; a worker's failure; a pause; cancel). `sentences_long.rs` 2. Existing cases: none changed.

**Tier 2 on the keyed `dist/`** (rebuilt): `pipeline-preview.spec.ts` and `rejections.spec.ts` whole, with `media-setup`: **32 passed, 1 skipped** (the existing skip), 4.4 min. Their code unchanged.

**Counts.** Rust 910 / 0 / 4 (908 + 2), Vitest 649 (638 + 11), `pnpm e2e` 57 (not re-run).

**Open:** nothing new. **Found, not fixed:** TS §43.6's `MAX_SENTENCE_MS` is not a rule of the code (above). The clip page's `REJECT_RESOLUTION` and `REJECT_FILE_SIZE` copy names V5's limits; with `high_res` on it must name 3,840 px and 16 GB (TS §43.2, "messages.ts picks by page and flag"): P179.

## Entry 34 - 2026-11-08 - P179: the interface

**Built.**

- `routes.tsx` (D-422): `ROUTES.long = "/app/long"`, behind `CapabilityGate`, with `long_video` off an explicit `<Navigate to="/app" replace>`.
- `ui/pages/LongPage.tsx` (§10.1): by `source-store`, the drop zone (with "Continue your video" when a reload left a source), the gate, the work, a refusal (`REJECT_DURATION` in the long page's words), a failure (Continue where it helps), the transcript; a short file goes to `/app`; a file handed over by "Make shorts from it" is taken on arrival; a cancelled source brings the drop zone back; a sign-in completed during the gate lets the gate ask again.
- `ui/layout/LongLayout.tsx` (D-423): the editor's bar (mark and account link), one column, the dark theme and the polite live region; no clip store.
- `ui/components/SourceGate.tsx` (§10.2), `SourceProgress.tsx` (§10.3: two stages with their bars, time left after 10%, "Keep this tab open", Cancel; paused, the reason and Continue, Sign in first when signed out; awaiting the file, a drop zone and "This is a different file"), `SourceTranscript.tsx` (§10.4); `ui/styles/long.module.css` (the editor's tokens only).
- `DropZone.tsx` (D-424, additive): `kind: "clip" | "long"` (default `clip`) and `onFile`; the long zone has no sample clip and no transcription card; under the clip page's stage zone, with `long_video` on, the quiet link "Got a long video? Make shorts from it". `RejectionPanel.tsx` (DC-1): with `long_video` on, `REJECT_DURATION` gains "Make shorts from it", its words unchanged. `EditorPage.tsx`: a file handed over by the long page opens as a clip with the toast "This one is short, so it opens as a clip."
- Copy: `copy/long.ts` gains the gate's "Sign in" (§1.3 names the button); `messages.ts` (678 of 700 lines) picks `REJECT_FILE_SIZE` (16 GB) and `REJECT_RESOLUTION` (3840) by `high_res` (TS §43.2), today's words with it off; `check-copy-codes.mjs` reads `copy/long.ts` and checks `SOURCE_REJECT_REASONS` and `SOURCE_ERROR_CODES`: 1 of 1, 2 of 2.

**What differs from the plan.** `LongLayout` does not mount `TopBar`, whose controls all act on a clip; it draws the same bar from the same styles. No user Pause button: §10.5 has no word for it and D-410 no reason for it; Cancel and the automatic pauses are there. The summary's word and sentence counts are not shown (§10.5 has no words for them). **The captures, axe and the self-review (G 9.2-9.4) move to P180**, whose specs drive every state of the page on `dist-flags/`; taking them twice would cost an hour of the night for nothing.

**Two fixes from the review before the commit.** (1) A file held by `import-clip` after a `REJECT_DURATION` was taken by the long page on any arrival: a person who chose Start over and later followed the link would have had that old file started on. "Make shorts from it" now navigates with `{ handOver: true }`, and the long page takes the handed file only then; any other is let go. (2) Waiting for the file had no way out: it gains Cancel (the machine's `awaiting_file → cancel`), so every working state has one.

**Tier 2 on the flags-off `dist/`** (keyed, rebuilt; the bundle's `long_video` and `high_res` both `!1`): a quick check (a scratch spec, deleted) that `/app` has no long link, `/app/long` lands on `/app`, and `rej_too_long_91s.mp4`'s panel has no new button: passed. `pipeline-preview.spec.ts` and `rejections.spec.ts` whole, with `media-setup`: **32 passed, 1 skipped** (the existing skip), 4.5 min. `pnpm e2e` (with `landing.spec.ts`): **57 passed**. Their code unchanged.

**Counts.** Rust 910 / 0 / 4 (no Rust change), Vitest 649, `pnpm e2e` 57.

**Open:** nothing new. **Found, not fixed:** none.

## Entry 35 - 2026-11-08 - P180: end to end, both builds, CI

**Built.**

- `tests-e2e/helpers/fake-api.ts` (D-431, additive): `transcribe.windows?: (index, total) => unknown` answers a request that carries `X-Offcut-Window`; without it every request gets `response`, as before. No other option and no default changed (CT G item 3).
- `tests-e2e/helpers/long-windows.ts` (new): each window's answer of `long_12min.mp4` from `fixtures/cloud/speech_60s_portrait.json` shifted by `n × 79.517 s + 0.960 s` (D-431 as read); `openLong`, `openClipSignedIn` (Free or Creator, a large file through the zone's own file field), the source stores' readers, and `capture` (G 9.2, only with `V61_REVIEW_DIR` set; never in CI).
- `playwright.config.ts` (D-394): `webServer` is two entries, `dist/` on 4173 and `dist-flags/` on 4174 (the second only where that build exists); the project `media-flags` runs `FLAG_SUITES` on 4174, one worker, after `media-setup`.
- The four specs: `route-long.spec.ts` 9 cases (the `/app` link and DC-1's button carrying the file; the empty page; the gate checking, then ready; the gate off; `long_12min.mp4` read in place and transcribed: 9 windows, each once with `X-Offcut-Window` and an Ogg body under 1 MiB, all 1,143 words with none lost or doubled, the first sentence, no request carrying any of 20 pieces of 32 bytes of the video; a 60 s file handed to `/app` and processed; `REJECT_DURATION`, `REJECT_RESOLUTION`, `REJECT_HDR` in their words; axe on every state). `long-resume.spec.ts` 6 cases (a reload while transcribing sends no kept window again; a reload during the audio step asks for the file, refuses another, and goes on from the checkpoint; 503 with the switch off pauses `unavailable`; 429 pauses `busy`; offline pauses `offline`; a lost session pauses `signed_out` and Continue opens the sign-in dialog; Continue finishes each; the tab title). `high-res.spec.ts` 4 cases (`rej_4k.mp4` taken at import; HEVC refused on the clip page, D-427 as read; above 3840 refused in words that say 3840; a 4K speech clip, made in the test from the reference, processed and exported by Creator at 1080 × 1920 with all 11 checks). `flags-identity.spec.ts` 1 case (below).
- CI (§12.1 as read): the `ci` job builds `web/dist-flags` (`OFFCUT_FEATURES_ON=long_video,high_res`, the same test key) and uploads it; `e2e-media` downloads it and runs `pnpm e2e:media:flags --grep @no-gpu` after `pnpm e2e:media --grep @no-gpu`; its timeout 30 → 45 minutes.

**Found by the first browser run, and fixed (the long page had not run in a browser before P180).**

1. **Every window failed with `E_INTERNAL`** (P177): the asr worker takes one job at a time and refused the second lane's call as `Busy`. `transcribe-source.ts` queues its calls to the asr worker; the requests stay two in flight (D-409 as read).
2. **The gate asked again forever** (P179): its "a sign-in while waiting" effect ran on every return to `gated`. It now asks again only from `signed_out`.
3. **A switched-off server paused as `busy`** (P177): a `/healthz` without `cloud_transcribe` is a server that does not transcribe (the type's `serde(default)`); the check reads `=== true` (D-410 as read).
4. **Offline at the first window showed "Something went wrong in Offcut"** (P178): the asr worker's script is fetched at its first use. A window step that throws while offline now pauses `offline`; a `run-source-pipeline.test.ts` case added.
5. **The long page's bar sat at the bottom right** (P179): the editor's bar places itself in a grid area `top` that `LongLayout`'s grid did not name. Named.
6. **axe: scrolling regions with nothing to focus** (P179): the page scrolls instead of an inner region; the long column's cards grow with their content; the transcript's list, which does scroll, is focusable and named by its heading.
7. **"Transcribing 0 of 0" before the windows start** (P179): the count is known from the duration and shown.

**What differs from the plan.** `rejections.spec.ts` is **not edited at all**: DC-1's button is asserted by `route-long.spec.ts`' first case, and the three cases whose file the flags-on clip page takes (`rej_4k.mp4`'s refusal, DC-2; "start over", whose refused file is that 4K one; the 500 MB limit, DC-3) are left out of `media-flags` by its `grepInvert`, not by an edit. `flags-identity.spec.ts`: two exports of one build are not byte-identical (sizes 33,461,716 and 33,580,996 B; the same 1,800 frames; identical audio packets; video PSNR 50.8 dB average, 45.4 dB least), so the comparison is identical audio packets, the same frame count and size, and PSNR at least 40 dB, beside equal transcript, edit state and render-cache key; the flags-on build's database is at structure version 2 after start (D-419), the flags-off build's at 1 (D-394 as read). `high-res.spec.ts`' processed 4K clip is the reference scaled to 2160 × 3840 in the test (`rej_4k.mp4` has no speech). Large files go through the drop zone's file field (`setInputFiles`): `dropFile` sends the bytes in one DevTools message, which closes the page at about 80 MB. The gate's "checking" is held by delaying only the page's first `/healthz`, since the session's own waits for `/healthz` too.

**P179's G 9.2-9.4, done here.** Every long-page state captured by the specs at 1280 × 800 and 1920 × 1080 into `~/v6.1-review/` (outside git), axe 0 serious or critical on every state the specs drive, and the self-review against ON §5's rubric: round 1 found the bar misplaced, the scrolling regions, "0 of 0" and, on the over-45-minutes card, its title said twice (the card now leaves out a body that only repeats the title); round 2 read every capture again: **self-review, rubric passed**.

**DECLARED CHANGE** `fake-api.ts`' `windows` option (D-431): its two changed lines are the option's type, widened, and the answer, which takes `windows` only when given. **Tests:** no existing case edited.

**Runs.** FLAG_SUITES on `dist-flags/`: run 1 **34 passed, 1 skipped** (the existing `rej_h264_444` fixme, D-78), 8.2 min; run 2 (after the last fix) **34 passed, 1 skipped**, 7.8 min. Tier 3: `pnpm check` ok; Vitest 650; `pnpm e2e --workers=4` 57 passed (rebuilt keyed `dist/`). The Rust workspace and `pnpm fixtures` have not changed since P178 and P173: P182's Tier 4 runs them. `cloud-transcribe.spec.ts` on `dist/` (it uses `fake-api.ts`' transcribe option): 8 passed with `media-setup`, 7.1 min, unchanged.

**Counts.** Rust 910 / 0 / 4 (no Rust change), Vitest 650 (649 + 1), `pnpm e2e` 57.

**Open:** nothing new. **Found, not fixed:** none.

## Entry 36 - 2026-11-09 - P181: experiments and the walk

**How it was read.** A scratch Playwright spec (`zz-te.spec.ts`, deleted) drove the real flags-on build (`dist-flags/`) in Chrome, signed in through `fake-api.ts` (G Phase 11: no inbox at night). Files were chosen in the long page's own file field, so Chrome read them from the disk. Readings in `docs/v6.1/experiments.md`.

**TE-17 (second read).** 45-minute 4K HEVC (1.48 GB) and 1080p H.264 (5.6 GB) read in place: probe 0.16 and 0.15 s, the audio step 22 and 55 s, peak page memory 34 and 37 MB; every pass line holds. **F-3's condition: 4K clip peak 883 MB: `high_res` may be turned on** (an 89 s 4K H.264 clip on `/app`, processed in 97 s and exported by Creator in 52 s; the founder's morning step).

**TE-18 (second read).** 40 minutes through Groq by the product: 31 requests (30 × 200, one 429 that paused the job `busy`; Continue finished it), p50 0.72 s and p90 1.05 s a window, 33 s of windows when not limited. At the 29 seams (a second run at 01:57 with the words kept): 4,677 words, 96 edits against the first loop (2.1%), **5 within 2 s of a seam (2 lost, 3 changed, none doubled)**, the base rate of edits elsewhere; no word in room tone. §13's "no word lost or doubled at a seam" fails by 2 lost words: stop-and-ask, recorded as open item 27; it does not block a dark merge.

**TE-20 (simulated).** A reload mid-run lost only the window in flight (10 requests for 9 windows; no kept window sent again). The hidden tab and a frozen page could not be simulated on the automated Chrome (it stayed `visible` minimised, and a visible page is not frozen): the real test is the founder's morning step.

**TE-21.** On `rej_hdr_hlg.mov` only (A-2): Chrome maps the HLG frame into sRGB without washing it out (levels and highlights match ffmpeg's straight decode) but clips saturated colours to black (49% of channels at 0 against 32%). `hdr_ok` stays `false` in every committed build.

**The scripted walk** (ON §5, P181 step 5): every row of §1.3's table captured into `~/v6.1-review/walk/` (outside git) and read with the Read tool: the `/app` link; the too-long panel with "Make shorts from it"; the gate signed out; cloud off; the line on what is sent; "Transcribing 3 of 30", "About 2 minutes left" and "55% - Offcut" in the tab; a 60 s file opened on `/app` with "This one is short, so it opens as a clip."; "Continue your video"; a pause with Continue; the transcript. Each says what is happening and what to do next.

**Every real Groq call** (F-1, A-3; the reference clip's looped audio only; the key read alone from `.env.deploy`, never printed): (1) 23:51-00:21, stopped: its calls were not recorded, at most 2,690 s (experiments.md says what that means for A-3); (2) 00:54-00:56, 31 requests, 30 answered, **2,690 s**; (3) 01:57-01:59, 31 requests, 30 answered, **2,690 s**. Groq's own limit (7,200 s an hour) was never reached; the 25% stop never fired.

**Found, not fixed.** `MAX_SOURCE_DURATION` is exact: `long_45min_4k_hevc.mov`, whose video track lasts 45:00.067, is refused as longer than 45 minutes. A phone's "45:00" recording can run a fraction of a second over; a tolerance (as the frame rate has) is the founder's call. `measureUserAgentSpecificMemory` counts the page and its workers, not Chrome's GPU process, so the decoders' own memory is not in these numbers; Chrome's task manager by hand is the founder's (R1).

**Counts.** Rust 910 / 0 / 4 (the workspace, run in this prompt), Vitest 650, `pnpm e2e` 57; the full media run on `dist/` (P182's Tier 4, run here while Groq's hour refilled): **112 passed, 1 skipped**, 40.5 min, the floor exactly. No product code changed in P181.

**Open:** **27 (new)**, the 2 words lost near seams; 19 and 24 stand, re-read.

## Entry 37 - 2026-11-09 - P182: copy-back, the two runs, ready to merge

**The founder merges** (their instruction of 2026-11-08: "do everything and tell me to merge, don't merge yourself"). So this prompt ends at a green, ready pull request; the merge, the `main` run and the tag follow when the founder merges, then §12.2's live checks and the closing entry.

**Copy-back (§18), each marked "(V6.1, as built)".**

- TS §5 (rows as files came; `flags-identity` and `route-long` with hyphens), §7 (V6.1's lint edges; `transcribe-source.ts` the second use-case that sends audio; the P-1 row), §22.1 and §24.5 (the window header, the two groups, the IP check in the handler, the log field), §22.8 (the two budget variables), §23.3 (structure version and record version; the `VersionError` rule), §39.3 (item 34: the long path's assumptions as measured), §41.3 (flag defaults, `OFFCUT_FEATURES_ON`, the two builds), §43.1-§43.9 (the rules in their real order; the types beside V5's pinned ones; `SOURCE_LIMITS`; reading in place; storage version 2; `audio-region.ts`; the windows at 10 s overlap, 34 for 45 minutes; the failure rules as built; the machine as built; the copy in `copy/long.ts`), §53.1 (TE-17, TE-18, TE-20, TE-21 read), §53.5 (the three events: not built, open item 26).
- PS §10.1 (L0-L4 as built, dark), §20.2A (the budgets read). BP §0 (V6.1's line), §8A.1 (as built; `quota.ts` eviction to V7, REOPENED BP §8A.1).
- The plan's §17 exit checklist ticked, the founder's boxes named.

**Tier 4 (the two runs and the rest), on the final code.** Rust workspace **910 / 0 / 4**; Vitest **650**; `pnpm e2e` **57**; the full media run on `dist/` **112 passed, 1 skipped** (40.5 min; the protected path as production runs it); `FLAG_SUITES` on `dist-flags/` **34 passed, 1 skipped** (P180's second run, after the last code change); `pnpm fixtures` up to date with both manifests. No product code changed after those runs: P181 and P182 changed documents only.

**ON §7.2's gates, with their evidence.**

- [x] Locally: `pnpm check` ok; Rust, Vitest, `pnpm e2e` as above; `pnpm build` ok with `check-hosts.mjs` ok (one host, the asset host, as in the CSP) and the bundle's `long_video:!1, high_res:!1`; counts at or above Setup's baseline (863 / 0 / 4, 568, 57).
- [x] The full `pnpm e2e:media` on `dist/` passed once: 112 passed with the one existing skip, the floor.
- [x] `FLAG_SUITES` on `dist-flags/` passed (`route-long`, `high-res`, `long-resume`, `flags-identity`, `rejections`), twice.
- [x] G 0.4's frozen check prints nothing; the additive check prints only its named lines (the handlers lines; `db.ts`' `open()` body, D-414; `transcribeAudio`'s signature, D-413; `dropSilentWords`' signature and level line, D-407; `route.rs`' moved steps, D-411; and two import lines of `asr.worker.ts`, P177).
- [x] `git diff --name-only $BASE...v6.1-build -- web/tests-e2e/`: the four new specs, `long-windows.ts` and `fake-api.ts` (its one option); `rejections.spec.ts` unchanged (no edit was needed); `server/tests/` adds cases only (0 removed lines).
- [x] `git diff --stat $BASE...v6.1-build`: 104 files, the plan's §4 files, their tests, docs, the fixtures scripts and the two workflows.
- [x] Goldens and snapshots: `git diff --name-only $BASE -- '*.snap' '*.png'` empty.
- [ ] The CI run on the pull request's head: push 3 below; read before the founder is told to merge.
- [x] The self-review rubric passed (P180).
- [x] `git grep -nE "gsk_[A-Za-z0-9]{20}"` empty; nothing private staged (no `.env*`, `testclips*`, `fixtures/.gen*`, `web/dist*`, no `zz-` file).
- **Time (A-1):** the founder merges when they choose; the deploy is watched by the agent after it.

**After the founder merges (the agent, ON §7.3):** the `main` run (`ci`, `e2e-media`, `deploy-api`, `deploy-web` with its smoke); `/healthz` with the merge's version and `cloud_transcribe: true`; `/api/v1/me` 401 with `Cache-Control: no-store` (D-429); an unauthenticated `POST /transcribe` with `X-Offcut-Window: 0/2` 401; `@smoke` against the live site; `/app` with no long link and `/app/long` redirecting to `/app`; then the tag `v6.1` on the merge commit (the founder's, or the agent's at their word), and the closing entry.

**For the founder, in the morning (ON §8).** (1) F-3: `high_res` may go on (TE-17's 4K clip read 883 MB); Vercel's `OFFCUT_FEATURES_ON=high_res` (Production), then re-run the `main` run's `deploy-web` job; never the Redeploy button. (2) TE-20's real test: a long video on `dist-flags/` through a five-minute sleep and a background tab. (3) Open items 19, 24 (Groq's Dev tier, or one window in flight, before `long_video` goes on), 25, 26, 27 (2 words lost near seams). (4) Found, not fixed: `MAX_SOURCE_DURATION` has no tolerance (a 45:00.067 file is refused). (5) `long_video` stays off until V6.4's alpha (F-4).

**Counts.** Rust 910 / 0 / 4, Vitest 650, `pnpm e2e` 57, media 112 + 1 skipped, `FLAG_SUITES` 34 + 1 skipped.

**Open:** 27 (P181). **Found, not fixed:** as above.


## Entry 38 - 2026-11-09 - Spec revision v1.5: the Submagic benchmark as rules, the picture work Tooscut showed, the Enhance panel (outside a prompt)

A planning session after V6.1's merge. Nothing in the code changed. The founder's rule: **Offcut survives only if its shorts match or beat Submagic's; skip what is very hard or a big change with low impact, keep anything that makes the output much better.** The agent read the six Submagic promos in `sh2clips/documents/submagic_clips/` frame by frame (only the "AFTER" halves of four show real Submagic output; one is an AI-generated presenter), then Tooscut's colour grading, transitions, keyframes and audio effects, and put the decisions to the founder as multiple-choice questions with a recommendation each. The three core documents are now v1.5; every change is marked "v1.5" where it sits.

**What the benchmark showed.** A visual change about every 2 s (one output: 11 changes in 23 s); every shot slowly pushing in; whooshes and pops on each B-roll or card entry over a quiet music bed (-16.3 LUFS); captions plainer than expected (one to four words, a heavy white sans with a soft glow, the keyword coloured with an emoji under it, punch words about 2x in capitals, hard swaps); a mixed-size hook lockup; full-screen stock B-roll; the 9:16 crop following whoever speaks in two-person podcasts. Submagic's outputs keep the source's colours unchanged. **The gap is not the captions** (Bold is near parity, and Offcut is likely ahead on line breaks, safe areas and timing) **but motion, sound, B-roll and following the speaker.**

**Founder's decisions (2026-11-09).**

| # | Question | Decision | Replaces |
|---|---|---|---|
| 1 | Order | **Keep the release numbers;** put each addition into the release that already holds what it needs; build V6.6 as V6.6a "Picture" and V6.6b "Graphics" | BP §8A.0's order, extended; "look before the Director" was declined |
| 2 | Following the speaker | **In, at V6.5:** mouth movement matched to speech, no new model; switch only at sentence starts; split when unsure or both speak; only in shots with two or more faces | PS §9.2's "one speaker at launch"; TS §46's rejection of speaker detection |
| 3 | Motion | **On by default, by rules,** on talking-head shots, alternating in and out; the plan's zooms on top; one switch | TS §47.2's "zoom only where the plan asks" |
| 4 | Switching enhancements off (the founder's request) | **Measured first, one switch each, remembered:** an Enhance panel with Voice cleanup, Motion, Colour & light, Sound effects, Music; the last choice starts the next video; Voice cleanup off keeps the loudness levelling | Voice chain always on (PS §8 principle 3) |
| 5 | HDR | **Keep rejecting HDR** with advice; no tone mapping | TE-21's open question; the agent recommended testing real phone clips first |
| 6 | Sound effects | **Synthesize our own** in code | TE-16's licensed bundle |
| 7 | Music | **The person's own track only;** no library at launch | TE-16's bundled tracks; the agent recommended buying a small licensed pack |
| 8 | Captions | **Four display fonts, six presets** with our own names, word pages, glow, punch words, emoji under the keyword | |
| 9 | Cards B and part D | **Keep both** before launch | The agent recommended moving them after launch |
| 10 | Quality check | **Rules fix what they can, CI checks the corpus,** never blocks an export | |
| 11 | Founding release | **After V6.8,** behind a blind test against Submagic (E-16) | BP §8A.6: right after V6.6 |
| 12 | V6.9-V6.11 | **Before launch, built while the founding offer sells** | Unchanged from Entry 0 |

**The agent's design choices, for the founder to confirm or change (open item 28).**

| # | Choice | Why |
|---|---|---|
| 1 | The quality work is a set of rules in the scene and the DSP (motion track, sound cues, quality check), not plan items the model must remember | The worst short must be good; a model forgets |
| 2 | `Enhance` lives in `EditState` with `#[serde(default)]` = `Enhance::v5()`, and `Enhance::v5()` gives V5's frames and samples (DC-16, INV-33) | A stored clip reopens exactly as saved; `flags_identity.spec.ts` keeps its meaning |
| 3 | Both voices (`out48` Clean and `lvl48` Level) are prepared, so the switch only re-runs `assemble` | The switch is instant in the preview |
| 4 | Motion: a 5% drift per segment, segments split at 4 s, never on screen recordings, scales multiplied and capped at 1.35 | Reads as a camera operator, never as a zoom effect |
| 5 | Speaker: a switch is a hard cut at a sentence start, at least 1.5 s per turn; the split stacks the two faces, left person on top; the Framing panel offers Follow, Show both, Show one | An editor cuts, never pans, between speakers |
| 6 | The grade is one uniform in the video pass (`Video.grade`), so a graded list stays on the simple path; `None` keeps the old pipeline byte for byte (DC-18) | No second render path for colour; goldens untouched |
| 7 | Grade bounds: ±0.5 EV, face light at most +0.3 EV, vignette at most 12%, saturation weighted away from skin; no colour controls for the person | Do no harm is the pass line (TE-29) |
| 8 | Caption glow is drawn as faded strokes in Vello, not a blur | Captions stay on the simple path |
| 9 | Six synthesized sounds; rules cues are not plan items, at most 12 per short and one per 0.8 s | Switching an item off removes its sound with it |
| 10 | Voice tuning: denoise by measured noise floor and a three-band EQ bounded to ±3 dB; no gate, reverb or de-esser yet | A good microphone is left almost alone |
| 11 | Quality fixes are `PlanOp::AddItem` with `origin: QualityFix`, so V6.8 builds `AddItem` early | Each fix is visible, switchable and undoable |
| 12 | V6.6b builds parts in the order A, C, E, B, D | The benchmark weighs captions above more cards |
| 13 | Tags `v6.6a` and `v6.6b`; a new milestone id M1.23 for Picture | Numbers and tags stay readable |
| 14 | The Creator bitrate (8 Mb/s) is re-read in the quality pass under TE-4, not changed now | Constant motion is harder to encode; a change needs a §41.2 row |
| 15 | B-roll takes the same drift; joins gain `ZoomIn` and `ZoomOut` (from Tooscut's transition types), each with a whoosh | |

**Learned from Tooscut, in our own design (TS §53.4).** Its colour model (lift, gamma and gain wheels; slope, offset and power) becomes the automatic grade's parameters; its bezier keyframe easing becomes `easing.rs`; its `ZoomIn`/`ZoomOut` transitions join §48.3; its three-band EQ (low shelf, peak, high shelf) becomes voice tuning. Not adopted: log colour spaces, its simple tone mapping (HDR stays rejected), LUT looks, HSL and power-window controls, rotate, flip and wipe transitions, reverb, a noise gate. The no-copy rule is unchanged.

**Changed.**

| File | What |
|---|---|
| `docs/technicalspec.md` (v1.5) | Status and §0; DC-8 widened from V6.5; new DC-16 (Enhance), DC-17 (speaker split), DC-18 (`Video.grade`), DC-19 (voice tuning); §41.3 flags `motion`, `speaker_framing`, `picture`, `voice_tune`; §41.5 rows for V6.2, V6.5, V6.6a, V6.6b, V6.7, V6.8; §42 the V6.6a/V6.6b note; §40.2 `Emphasis.size`, `Emoji.place`, `HookTitle` to V6.2; §40.10 rows; §46 model line, §46.1's extra outputs, new §46.6 following the speaker; §47 retitled with the quality bar; §47.1 own music, synthesized sounds, rules cues, voice tuning; §47.2 motion by rules and easing curves; new §47.3 captions, §47.4 picture, §47.5 the Enhance panel, §47.6 the quality check; §48.3 zoom transitions, §48.4 the blur's source; §49.2 `AddItem` in V6.8 with `OpOrigin`; §28 milestones M1.9-M1.19 and new M1.23; TE-16 and TE-21 closed; new TE-27, TE-28, TE-29; INV-33; §53.4 Tooscut rows; §53.5 `export_done` fields; §53.6 items 12-15; the founding release after V6.8 |
| `docs/product.md` (v1.5) | Title and positioning; ICP and "not targeted"; principle 3 (the Enhance panel); §9.1 flows; §9.2 rows (music, two people on screen, HDR); §9.2.1 voice cleanup off; §9.4 HDR and speakers; §9.5 rows for V6.2, V6.5, V6.6a, V6.6b, V6.7, V6.8 and the founding release; §9.7 rows; §12.8; §12.10 two people on screen; §12.12 zoom transitions; new §12.16 the finished look and the Enhance panel; §16 `MusicTrack`; §13, §18 and E-5 timing; new E-16 (the blind test); §21 items 20, 22 (HDR), 23 (music library) |
| `docs/buildplan.md` (v1.5) | Status; a v1.5 paragraph; the version table (V6.6a, V6.6b); §8A goal and order; the founder's table (two-person set, TE-29's panel, E-16 after V6.8); the estimate; §8A.2, §8A.5, §8A.7, §8A.8 additions; §8A.6 rewritten as 8A.6a Picture and 8A.6b Graphics; the founding release and its boxes moved after V6.8; §8A.12's v1.5 inputs; V6's founding-release mentions, §11.3, §12's boxes, appendices A and B |
| `docs/v6/v6changelog.md` | This entry; open items 21 (closed), 28-31 |

**Estimate.** 154 prompts still to build (V6.2 19, V6.3 16, V6.4 14, V6.5 14, V6.6a 9, V6.6b 21, V6.7 10, V6.8 21, V6.9-V6.11 30): **+30 against v1.4,** about 6-8 weeks at the founder's pace. Public launch about February to early March 2027 (ESTIMATE). TE-27 or TE-29 at twice its estimate adds up to about 11 prompts.

**Checked.** Documents only: no code or test changed, nothing built. Every edit's anchor matched exactly once, or the script wrote nothing. Every new `§` reference (§46.6, §47.1-§47.6, §8A.6a, §8A.6b, §8A.8, §12.16, PS §21 items 22-23, E-16, DC-16 to DC-19, TE-27 to TE-29, INV-33, M1.23) resolves to a heading or a row. `node scripts/check-file-tree.mjs` passes.

**Not done.** Not committed: the founder asked for the documents. `docs/v6.2/v6.2implementation.md`, its guide and prompts come next, after the founder confirms open item 28; as before, each release's plan adds its new files to TS §5's tree. V6.1's tag and closing entry still wait (the V6.1 status of Entry 37).

## Entry 39 - 2026-11-11 - V6.1 closed: merged, deployed, tagged; v1.5 confirmed; V6.2's branch (outside a prompt)

**Merge.** The founder merged pull request 16 as `b73fea5` on 2026-11-09 (03:35 UTC), with the same tree as `628c995`. Tagged `v6.1` on `b73fea5` and pushed by the agent at the founder's word (2026-11-11).

**The `main` run, 42557223857.** All five jobs green on the first attempt (`ci`, `e2e-media`, `deploy-api`, `deploy-web` with its smoke).

**ON §7.3's live checks (agent, 2026-11-09).** `/healthz` through the site reads `b73fea5` with `cloud_transcribe: true`; `/api/v1/me` without a session 401 with `Cache-Control: no-store` (D-429); an unauthenticated `POST /transcribe` with `X-Offcut-Window: 0/2` 401; `@smoke` against the live site green; `/app` shows no long link and `/app/long` redirects to `/app` (`long_video` off). The only console noise is the signed-out `POST /auth/refresh` 401, which predates V6.1 and is not a regression. Screens kept outside git.

**Flags in production.** `long_video` off until V6.4's alpha (F-4). `high_res` may go on at the founder's choice (F-3, Entry 37); not read here.

**v1.5 confirmed.** The founder confirmed the agent's 15 design choices of Entry 38 (2026-11-11): open item 28 is closed, and the three core documents at v1.5 are committed with this entry, unchanged since Entry 38.

**V6.2's branch.** `v6.2-build` from `main` at `b73fea5` (BP §8A.0 step 1); this entry is its first commit. V6.1's narrow permission rules (ON A-5) are removed from the git-excluded `.claude/settings.local.json`. `docs/v6.2/`'s plan, guide and prompts are not written yet, at the founder's word; Setup's baseline of TS §41.1's proofs is read when they are.

**Open after V6.1.** For the founder: F-3 (`high_res` in production); TE-20's real sleep test; open items 19, 24 and 27 before V6.4's alpha; 25 (V6.3) and 26 (V6.4); 16 and 17 from V6. Found, not fixed: `MAX_SOURCE_DURATION` has no tolerance (a 45:00.067 file is refused).

**Final counts (Entry 37).** Rust 910 / 0 / 4, Vitest 650, `pnpm e2e` 57, media 112 + 1 skipped, `FLAG_SUITES` 34 + 1 skipped.

## Entry 40 - 2026-11-11 - V6.2's plan, guide and prompts (outside a prompt)

**The founder's decisions (2026-11-11).** (1) **Gemini is the only AI provider;** the key is in `.env` and `.env.deploy`. TE-15 therefore compares two Gemini models, not two providers; the `PlanProvider` trait stays, with one adapter. (2) **V6.2 is built carefully, by hand, one prompt at a time:** "this version is very crucial because further versions depend heavily on it; I don't care if the number of prompts increases." No overnight run; risky parts are split so each proves one thing before the next is built on it; four founder checkpoints.

**Written (uncommitted until V6.2's Setup).** `docs/v6.2/v6.2implementation.md` (D-433 to D-499), `docs/v6.2/v6.2buildguide.md`, `docs/v6.2/coding-promptsv6.2.md` (Setup and P183-P212: 30 prompts against BP §8A.0's 19), and their TS §5 rows. Read against the code at `aef4a1a`.

**What the reading of the code changed against the specs** (each marked REOPENED in the plan, copied back in P212):

- `offcut-scene/src/motion.rs` is V4's caption motion, so the camera drift goes in a new `camera.rs` (TS §47.2 named `motion.rs`); `easing.rs` gains the bezier beside its existing `Easing` enum.
- The resolved params of the new kinds go in `EventParams`, where V4's `ValidItem` keeps them (TS §40.2 lists them only as wire params).
- `HookTitle.accent` is an index into the hook's own words, not a transcript `WordIdx`.
- The `plans` store is IndexedDB structure version 3 (V6.1 took 2); the unused `events` store is kept, not removed (TS §40.8).
- `EditState.enhance` is skipped when it equals `Enhance::v5()`, so stored records and the render-cache key are unchanged for every existing clip; `Enhance` has two fields in V6.2 (`voice`, `motion`), and `EnhanceReport` waits for the first stage that adapts to a measurement (V6.6a, V6.7).
- TS §47.3 gives the glow to none of V6.2's three styles, so nobody would see it: F-3 proposes a fourth style, Glow, leaving Clean, Bold and Mono and their goldens as they are.
- Images are drawn in the Vello overlay rather than a new compositor pass, so the video path is untouched; the emoji and icons are embedded in the render binary as the fonts are.
- `privacy-network.spec.ts` is V8's; V6.2's network rules are asserted in a new `ai-plan.spec.ts`.
- Nine existing tests pin V4's catalog and V6's lists on purpose; the plan names each one that V6.2 widens (D-440), and allows no other.

**Open.** Item 32 (F-1 to F-9) before Setup. Open item 1 (`AI_MONTHLY_CAP_USD`) is F-6; open item 2 (provider and model) is now Gemini, the model chosen at CP 1 (P188).

**The founder's answers (2026-11-11, later the same day).** "Yes to all", except: billing on the key's Google project was **already on** (F-2); **TE-15 reads 15 labelled transcripts, not 31** (D-493: `a01` and `w01`-`w14`, 60 calls; `a02`, a private recording's transcript, is never sent); **no AI plans for signed-out people, only for signed-in ones** (D-494): `/plan` requires a bearer and there is no anonymous quota, which reopens TS §40.7's "Bearer optional" and §52's anonymous row (copied back in P212). Free 40 plans a month and Creator 1,000 soft stand; the cap is 50 USD (F-6). Open item 32 is closed; open item 1 is answered (set on Render at the merge).

**A review of the three documents (2026-11-11), and the founder's answers.** An independent review checked the plan against the code (every fact it sampled held) and found five problems. The founder answered yes to all but one, and the plan records them as D-495 to D-499:

- **The emoji and icon names come before the catalog** (D-495): P184 fixes the 72 names (CP 2a), so TE-15 tests the real prompt; P195 makes their images (CP 2b).
- **Real recordings in TE-15: declined.** "Gemini provides fine models": TE-15 stays on the 15 transcripts of D-493.
- **CP 1 reads ten plans as annotated transcripts; CP 3 is the whole reference clip with its real plan, exported in the browser and watched beside the Submagic benchmark** (D-496). The review asked for a native render; the repository decodes video only in the browser, so the export is the browser's.
- **`motion` stays off in production until V6.5** (D-497), because nothing before V6.5's shots can tell a screen recording apart (Entry 38, choice 4).
- **Latency** (D-498): p90 ≤ 10 s targeted, 20 s the hard line, a 20 s client timeout, 15 s per provider call; the Flash-class model unless Pro's plans are clearly better.
- **Process** (D-499): P187 joins P186 and P198 joins P199 (28 runs, numbers kept); Medium effort where goldens guard and on records, High kept on punch words, emoji and the hook; a fresh session per prompt, cited sections only, trimmed test output, entries of about 30 lines; Setup adds `scripts/v62-gate.sh`, `debug = "line-tables-only"` for dev and test builds, and `rust-lld` for the Windows target only.

## Entry 41 - 2026-11-11 - V6.2 Setup: the documents, the key's name, the gate, the build settings, the floor

**Branch.** `v6.2-build` at `aef4a1a` on `main`'s `b73fea5` (G 0.2 checked). Built by hand, one prompt at a time (D-434). This commit adds `docs/v6.2/` (plan, guide, prompts, `experiments.md`), Entries 40-41 and TS §5's rows.

**Founder's answers, as recorded** (Entry 40; the plan's D-442, D-493 to D-499): F-1 to F-9 yes, with billing already on, TE-15 on 15 transcripts, AI plans for signed-in people only; the review's points yes, except real recordings in TE-15; Motion off in production until V6.5.

**The key (F-1).** `GEMINI_API_KEY` renamed `PLAN_API_KEY` in `.env` and `.env.deploy` (one line each, value untouched, never printed); `.env` gains `PLAN_ENABLED=true`, `PLAN_PROVIDER=gemini`, `AI_MONTHLY_CAP_USD=5`. Both files stay git-ignored. Google's model list answers the key with HTTP 200 (no content sent). **Found:** the key is not in the `AIza…` form, so a pattern alone would not catch it in a file; the gate also searches for the exact values of `PLAN_API_KEY` and `TRANSCRIBE_API_KEY` from `.env`, never printing them, and skips values under 20 characters (the local `TRANSCRIBE_API_KEY` is a 6-character placeholder, which matched ordinary words).

**Build settings (G 0.8).** Root `Cargo.toml`: `[profile.dev]` and `[profile.test]` with `debug = "line-tables-only"`; `[profile.release]` unchanged; no `opt-level` change. `.cargo/config.toml` (new): `rust-lld.exe` for `x86_64-pc-windows-msvc` only. Kept: from a clean `target/`, `offcut-render`'s golden tests built and linked in 89 s and `offcut-api` in 90 s. `target/` was 20.5 GB before (cleared with `cargo clean`); 6.2 GB after the whole workspace's tests.

**The gate (G 0.9).** `scripts/v62-gate.sh` (TS §5 row added): branch, `zz` files, key patterns and values, the entitlement test key unset, G 0.4's frozen paths (changed or new files), no modified golden or snapshot of the base, the additive check against an allow list kept in the script (empty now; a prompt that must remove a line adds its pattern with the D-number), nothing private tracked, `check-file-tree.mjs`, and no `ignore = "V6.2` once a P206 commit exists. Self-tested: planted `zz` files, a key pattern, a frozen-file edit, a removed line in `easing.rs` and a changed golden each failed it; restored, it answers `gate: ok`.

**The floor, with the new settings (D-438; V6.2's baseline, BP §8A.0 step 1).** Rust **910 / 0 / 4** (421 s); Vitest **650**; `pnpm e2e` **57**. Media 112 + 1 skipped and `FLAG_SUITES` 34 + 1 stand from V6.1 (not re-run: D-437).

**Open.** Nothing new. Next: P183, read before building.

## Entry 42 - 2026-11-11 - P183: read before building

No product code changed: two temporary probes ran and were removed, and two renderer files patched for a probe were restored byte for byte. Readings are in `docs/v6.2/experiments.md`; the plan gains "As read (P183)" notes under D-457, D-459, D-460, D-463, D-470, D-471, D-472, D-492 and §10.1, TE-31's pass line, and a row P183b.

**Gemini (TE-15 first read).** The key sees 62 models; the candidates are `gemini-3.8-flash` and `gemini-3.1-pro-preview`, with `gemini-4-argon` a third for CP 1. Prices and terms read from Google's pages: paid-tier prompts are not used to improve products, are kept 55 days for abuse checks with possible review by Google staff; developer logs are on by default in new projects and can be turned off.

**Real calls (F-9: `a01` only).** (1) `gemini-3.8-flash`, the `@v1` texts and schema: 200 in 22.7 s; 4,782 in, 937 out, 1,557 thinking tokens; about 1.3 cents. Gemini accepted the canonical schema but ignored `const` (`format` and `catalog` came back as `{}`); with the server setting both, the plan parsed and `validate` kept all 7 items. (2) `gemini-3.1-pro-preview`: **429, free-tier quotas only**. Plus Setup's and P183's two free model-list calls.

**Stop and ask (open item 33): the key's project is on the free tier.** On it Google may train on what is sent and humans may read it, so no further call is made until the founder confirms billing. P183b takes the remaining reads (Pro-class, Flash with less thinking) before P188; P184 to P187 do not need them.

**Decisions changed by the readings.** The server sets `format`, `catalog` and `source`; the model never writes them (D-457, D-460). Thinking is on by default and makes the call slow: `PLAN_THINKING` from configuration (D-459). Prices change on 2027-01-01 (open item 34). Straight alpha needs no premultiplying (D-472).

**Vello's images (TE-30 first read, WARP).** `Scene::draw_image` with peniko's `ImageBrush` (straight alpha, bilinear, an opacity multiplier) through the real `render_to_image`: exact colours at 0.5×, 1× and 1.5×, the exact half-opacity blend, no dark fringe, soft edges, clean clipping.

**Licences.** Fluent Emoji: MIT, notice only, pinned at `1ffb34c`; 3D PNGs are 256 px RGBA (about 550 kB for the bundle at 160 px, under D-471's 600 kB). Lucide: ISC, already in `web/LICENSES.md`; icons rasterised with Playwright's Chromium (no new package).

**Drift (TE-31 first look).** A linear 1.00 → 1.05 drift through the real video pass moved a shape 0.021-0.039 output px every frame (0.030 expected), never zero then a jump: no snapping, so the pass line becomes "no stepping", not a speed limit.

**Counts.** Unchanged (no product code): Rust 910 / 0 / 4, Vitest 650, `pnpm e2e` 57 (Entry 41).

**Open.** 33 (billing, logs), 34 (prices on 2027-01-01). Next: P184 (the types, CP 2a); P183b once billing is confirmed.

## Entry 43 - 2026-11-11 - P183b: DeepSeek's reads (the founder's provider)

**Founder's decision (D-500, REOPENED D-435):** the paid Gemini account is blocked, so **DeepSeek is the only AI provider, with thinking always off.** The adapter becomes `deepseek.rs`. The three V6.2 documents gain one rule: where they say Gemini, read DeepSeek. P183's Gemini readings stay as history. No product code changed.

**Read** (`experiments.md` TE-15, "The founder's decision"): models `deepseek-flash` (V4.1 Flash) and `deepseek-v4-pro`; prices per 1M tokens at peak: Flash 0.006 / 0.30 / 1.20 (cache hit / miss / output), Pro 0.044 / 1.32 / 3.96, off-peak half; the API, JSON-mode and thinking docs; the privacy policy and the Open Platform terms.

**Real calls (F-9: `a01` only).** Thinking off, `json_object`, the `@v1` texts with the schema in the system prompt. Flash ×3: 200 in 2.5-2.8 s, about 5,670 in / 460-520 out; `validate` kept 7, 9 and 7 items (drops: two `AnchorRule`, one `NumberUnresolved`). Pro ×1: 200 in 5.1 s, 5 of 5 kept. `json_schema` strict: **400, unavailable.** Before P183b, at the founder's request: thinking on took 35 s (Flash) and 145 s (Pro). Balance $1.99 → $1.93 over thirteen calls in all.

**Decisions changed by the readings.** No provider-shaped schema: the schema goes in the system prompt as text (D-457). No `PLAN_THINKING`: always disabled (D-459 → D-500). Blank content is a parse failure and takes the one retry. The spend counts input at the cache-miss price unless P191 adds a third price (D-463). `PLAN_PROVIDER=deepseek` (D-465).

**Terms, and what the product may say (D-492).** DeepSeek's terms do not say that API inputs are not kept or not trained on, and its privacy policy stores data in China. So the in-app copy names DeepSeek and China and makes neither claim. The landing page does not change (the founder: its promise is about the video). The privacy page and the settings table gain DeepSeek before `ai_plan` goes on (open item 35).

**First read against TE-15's lines:** parse 4/4, latency well under 10 s, cost about $0.0007 a Flash plan with the cached prompt. **Flash's median drop rate of 12.5% is over the 10% line on this one transcript;** P186's prompt prose and TE-15's 15 transcripts take it up.

**Gate.** `scripts/v62-gate.sh` also searches for `PLAN_API_KEY2`'s value and a DeepSeek key pattern.

**Counts.** Unchanged (no product code): Rust 910 / 0 / 4, Vitest 650, `pnpm e2e` 57.

**Open.** 33 and 34 closed (moot). New: 35 (the privacy page and the settings table name DeepSeek before `ai_plan` is on), 36 (rename `PLAN_API_KEY2`, before P189). Next: P184 (the types, CP 2a).

## Entry 44 - 2026-11-12 - P184: the types

**Founder's decisions for this night (2026-11-12).** P184 to P189 run in one session, overnight, instead of one fresh session each (bends D-499); P188 makes 30 DeepSeek calls, not 60 (each model once on the 15 transcripts); open item 36's rename is a yes; P189's default model is `deepseek-flash`; a small disagreement between documents follows the D-number with an "As read" note, while stored formats, privacy, money and promises still stop the run.

**CP 2a: the 72 names, approved by the founder as drafted (2026-11-12).** Each was checked against the upstream file list first (Fluent Emoji at `1ffb34c`, 3,146 "3D" PNGs; `lucide-static` 1.61.0, 2,171 SVGs), so P195 can fail only on looks. Each variant's doc in `assets.rs` names its upstream file. Emoji (48): laughing, heart_eyes, star_struck, mind_blown, thinking, shocked, crying, cool, partying, skeptical, eyes, angry, nervous; thumbs_up, thumbs_down, clap, raised_hands, muscle, folded_hands, handshake (the "Default" tone); money_bag, banknote, money_wings, gem, chart_up, chart_down, trophy; light_bulb, brain, target (Bullseye), crown, rocket; fire, hundred, sparkles, lightning, star, collision, party_popper; check, cross, warning, question, exclamation; alarm_clock, hourglass; heart, broken_heart. Icons (24): trending_up, trending_down, bar_chart, dollar, wallet, shopping_cart, briefcase, clock, calendar, lightbulb, target, check, cross, alert, arrow_right, heart, star, book, lock, shield, users, smartphone, laptop, mail.

**What changed.** `offcut-types`: `EventKind` gains `Emphasis`, `IconPop`, `PunchIn`, `Emoji`, `HookTitle` at the end; `EventParams` the five resolved arms and `ItemParams` the five wire arms, at the end (§6.1); `DropReason` gains `Density`, `HookMissing`. New `assets.rs` (`EmojiId`, `IconId`, each with `ALL`, `NAMES`, `name()`), `enhance.rs` (`VoiceMode`, `Enhance`, `v5()`, `is_v5()`, `defaults(motion_flag)`), `params.rs` (`Treatment`, `EmphasisSize`, `ZoomScale`, `ZoomMotion`, `EmojiPlace`). **REOPENED CONTRACT (D-446, D-485):** `EditState.enhance` with `#[serde(default = "Enhance::v5", skip_serializing_if = "Enhance::is_v5")]` and `#[ts(optional, as = "Option<Enhance>")]`, so a `v5()` state's JSON is byte for byte V6.1's and TypeScript sees `enhance?: Enhance` (no web code had to set it). **DECLARED CHANGE:** `offcut-wasm-render`'s `EditInput` reads `enhance` (default `v5()`) and passes it on; nothing reads it yet. Every new arm is "not drawn": the scene's `layout` returns `Ok(None)`; `validate` drops the five at V-2 (`UnknownKind`, no catalog row until P186). `pnpm gen:types`: `domain.ts` gains the nine types and the arms.

**Differs from the plan (As read notes under D-439 and D-440).**
- **`offcut-detect` is additive, not frozen.** It matches every `EventKind` by name and holds D-446's codes 4-8 (`event_id.rs`), so the kinds cannot be added with it frozen. Arms only: codes 4-8 in `event_id.rs` and `lib.rs`' order, a threshold of infinity, a hold of 0, never an overlay, priority 4 in `resolve.rs`. No line removed, no file added; the gate now checks exactly that. Its tests pass with their assertions unchanged.
- **Existing tests touched beyond D-440's nine.** (10) `events.rs`' `event_params_typescript_is_a_union_tagged_with_kind`: before `count(), 4`, after the four names kept, the five added and `count(), 9` (the gate's allow list names that one line). (11) An added arm, with no line changed, in each exhaustive `match` of a test file: `offcut-detect`'s four test modules and `tests/{precision_recall,rules_table,determinism,redetect}.rs`, `offcut-plan/tests/plan_from_rules.rs`, `offcut-scene/tests/{catalog_complete,timemap_identity}.rs`, `offcut-render/tests/golden_frames.rs`.
- **`params.rs`** holds the five param enums (§6.1 put them in `events.rs`), so `typescript_declares_every_type_and_limit_once` stays unchanged, as V6.1's `source.rs` did.
- **The web code had to name the kinds too:** `copy/plan.ts` (new, D-491) with the list's kind names (§12's drafts) and type-only feed lines, imported by `messages.ts` (680 lines); stand-in icons (Type, Sparkles, Crop) and switch cases in `PlanItemList.tsx`, `ProcessingFeed.tsx` and `run-pipeline.ts`'s `displayOf` (none reachable: the detector never makes the five).

**Tests.** New: 3 in `assets.rs`, 3 in `enhance.rs`, 5 in `edit.rs` (a `v5()` state equals V6.1's literal; a record without the field reads `v5()`; `motion: true` round-trips; a V6.1-shaped reader accepts a V6.2 record; the TypeScript field is optional). No golden or snapshot touched.

**Counts.** Rust **921 / 0 / 4** (910 before this prompt, re-read first); Vitest **650**; `pnpm e2e` not run (decision 8A: once, at the night's end). `pnpm check` green.

**Found, not fixed.** The list's draft names give "Emphasis" to both `keyword_pop` (V4) and `emphasis` (§12): one to rename at CP 3. TS §52 says the closed sets live "next to their drawing code"; D-447 puts them in `offcut-types`, as built.

**Open.** 35, 36 (36 answered yes; the rename is done in P189). Next: P185.

## Entry 45 - 2026-11-12 - P185: validation pieces

**What changed (`offcut-plan`).** `ParamTy::Index { max }` (0 to `max`), with its arms in `config.rs`, `validate_params.rs` and `prompt.rs` (text "a whole number from 0 to N", schema integer 0..N); no row uses it yet, so the `@v1` snapshots are unchanged. **DC-7 (D-449):** V-1 refuses only a catalog newer than `CATALOG_VERSION`. **`ValidPlan.extras`** (`ValidExtras { hook, post_caption, hashtags }`, accessor `extras()`, exported) holds the extras that pass D-454; cuts, chapters and music stay `NotInThisBuild` (D-455). Per item: the five kinds resolve (the four word pointers copy through; `HookTitle` takes the valid hook's text, `HookMissing` without one, its accent below the hook's word count and within the row's `Index`, else `ParamsMismatch`, D-453); `ClipWide` anchors are exactly `0..word count`; a `PunchIn` spans 1-3 words (`Snap`) or 2-40 (`Push`) inside one sentence, else `AnchorRule` (D-450). After V-6, three passes in anchor order: the zoom rule (two `PunchIn`s never overlap, one a sentence: `TooClose`), the punch words (6 s from the last punch's end: `TooClose`; 4 a clip: `OverLimit`, D-452), and for a model's plan V-9 (one visual start in any 2.5 s: `Density`, D-451). `hold_ms`: `IconPop` holds the event zone as V4's visuals do, the other four hold none. The kinds still have no catalog row, so `validate` drops them at V-2 until P186.

**DECLARED CHANGE (D-440 (2)), each assertion before and after.**
- `tests/validate.rs` `case_20_not_in_this_build_each_non_item_field`: before, the hook, the post caption and `["editing"]` each gave one `NotInThisBuild`; after, each is kept with no drop (`extras().hook == "I almost quit"`, the caption, `hashtags == ["#editing"]`: a hashtag now needs its `#`). The five cuts, the chapter and the music keep their assertion.
- `tests/validate.rs` `the_non_item_fields_are_dropped_and_the_items_kept`: before `dropped.len() == 6`; after `3` (cut, chapter, music) and `extras().hashtags == ["#editing"]` (the input's hashtag gains its `#`).
- `validate.rs`' inline `every_non_item_field_is_dropped_in_v4`: before `8`; after `4`, with the hook, the caption and `["#editing", "#video"]` kept.
- **Beyond the plan's list:** `offcut-wasm-render`'s `a_field_v4_does_not_apply_is_dropped_and_reported` used the hook as its example of a field not applied; it now uses the music cue, with its assertion unchanged (the gate's allow list names the two removed lines). And `offcut-plan/tests/ui/valid_plan_private.stderr` (trybuild) was regenerated, as its driver says to: building a `ValidPlan` outside the crate still fails on private fields, and the message now names `extras` too.

**As read** (notes under D-453 and D-454, and D-440's (12)): a hashtag without `#`, empty or with another character is `ParamsMismatch`; a ninth hashtag `OverLimit`; a hook outside 2-9 words `LabelTooLong`. A gap is measured from the last kept item's end, as V-6's `min_gap` is; the density window from start to start.

**Tests.** New: `tests/validate_v62.rs` (5: the hook, the caption, the hashtags, a rules plan without extras, DC-7), inline `validate.rs` (4: the zoom rule, the punch words, V-9, DC-7, each on `ValidItem`s directly), inline `validate_params.rs` (2: the hook's checks, the zoom's span rule). `validate_rules_plans.rs` and `plan_from_rules.rs` unchanged and passing, so every rules plan of the labelled set still validates whole.

**Counts.** Rust **932 / 0 / 4**; Vitest **650**; no golden or snapshot touched. `cargo test -p offcut-plan`: 65.

**Found, not fixed.** `validate.rs`' inline `v1_another_format_or_catalog_refuses_the_plan` refuses `empty_rules(2)` as a catalog too new; with `CATALOG_VERSION = 2` in P186 it will be read. P186 must handle it (it is not on D-440's list).

**Open.** 35, 36. Next: P186.

## Entry 46 - 2026-11-12 - P186 (with P187): the catalog rows, the prompt and the schema, v2

**What changed.** **REOPENED CONTRACT (D-448):** `CATALOG_VERSION = 2` with §6.2's five rows after V4's four, all `Producers::Model`: `emphasis` (caption, Point, 12, 800 ms), `icon_pop` (event, Point, 6, 2,500 ms), `punch_in` (video, Words 1-40, 8, 1,500 ms), `emoji` (caption, Point, 6, 2,500 ms), `hook_title` (hook, ClipWide, 1, 0). Their `Enum` params list P184's final names (`IconId::NAMES`, `EmojiId::NAMES`) and the param enums' serde names; a new test proves each name deserializes. New zones `"video"` and `"hook"`, and the scene's `ZoneId` gains `Video` (the canvas) and `Hook` (the safe area), drawing nothing. The rules plan's `catalog` is 2. `pnpm gen:types`: `CATALOG_VERSION = 2` in `domain.ts`.

**The prompt's new paragraphs (D-456), generated after the kinds and before the rules, numbers read from `validate`:**
> A finished short: open with a hook title only if the first sentence does not already hook; zoom in on each sentence's key claim; never stack two visuals, so at most one visual (a number, a list, a from-to, an icon, an emoji or the hook title) starts in any 2.5 seconds; end on the payoff, not a call to action, unless the speaker asks for one.
>
> Zooms: zoom on the claim, the number or the punchline, never on filler. A snap covers one strong word (1 to 3 words); a push covers a sentence that builds (2 to 40 words). A zoom stays inside one sentence; a sentence has at most one, and two zooms never overlap.
>
> Emphasis and emoji: emphasis marks the one word that carries a sentence. Give size punch only to the line that lands hardest: at most one punch word in any 6 seconds, and at most 4 in a clip. Put an emoji under its word, or beside it, only where it says the word better than text. Use only the icon and emoji names listed above.
>
> Text extras, under extras: hook, a title for the short in the speaker's voice, 2 to 9 words and at most 48 characters, with no digit the speaker did not say. Write it always; add a hook_title item, whose accent is the index of the hook's strongest word counting from 0, only if the first sentence does not already hook. post_caption: the text to post with the short, at most 2200 characters. hashtags: at most 8, each # followed by 1 to 30 letters, digits or underscores.

Also: a `ClipWide` kind's anchors now read "the whole clip, written {"start": 0, "end": the number of words}" (validate takes exactly that, P185); the five rows' own sentences are §6.2's drafts word for word.

**The schema's changes (`output_schema@v2`).** `catalog` const 2; `extras.properties` gains `hook` (string, maxLength 48), `post_caption` (string, maxLength 2200), `hashtags` (array, maxItems 8, items string `^#[A-Za-z0-9_]{1,30}$`); `items.oneOf` gains the five arms, each with `kind` const, word-range anchors, required params (`Enum`s as `enum` lists, `accent` integer 0-8), `confidence` null. No provider-shaped schema: DeepSeek takes the canonical schema as text (D-457, D-500).

**Snapshots (new files, each read in full before acceptance).** `catalog_snapshot__system_prompt@v2.snap`, `catalog_snapshot__output_schema@v2.snap`, `catalog_snapshot__user_message@v2.snap` (its body identical to `@v1`'s, 190 lines: the user message is unchanged). The three `@v1` files stay on disk, unchanged and untested.

**DECLARED CHANGE (D-440 (1), (9)).** `catalog.rs`' two inline tests keep every V4 assertion and gain the rows (`the_rows_are_in_code_order_once_each`: nine kinds, four zone names; `the_rows_hold_the_numbers_of_the_plan`: V4's four are `Both`, the five rows' numbers and zones, the five `Model`). `catalog_complete.rs`' three cases carry `#[ignore = "V6.2: the five kinds are drawn by P206"]`. **Beyond the list (D-440 (13), As read):** the catalog's count or version pinned in `config.rs` (4 → 9), `prompt.rs` (4 → 9), `validate.rs`' V-1 case and `offcut-wasm-render`'s refusal case (newer catalog 2 → `CATALOG_VERSION + 1` and 3; the gate allows the one removed line), `offcut-types`' TS test (`CATALOG_VERSION = 1` → 2); and one match arm in `catalog_complete.rs` for the two new zones.

**Tests.** New: 8 in `validate_v62.rs` (each kind kept and resolved; a rules plan of catalog 2 refuses them; the hook title's four drops; a word outside its anchors for the three word kinds; V-3; D-452; V-9 with the hook title, and a rules plan exempt from V-9 where a model's same plan is not), `catalog.rs`' enum names, `prompt.rs`' v2 kinds and extras read as an `EditPlan`.

**Counts.** Rust **939 / 0 / 7** (the three ignores are D-440 (9)'s); Vitest **650**; `cargo test -p offcut-plan` 75. Every scene snapshot and golden unchanged (gate).

**Found, not fixed.** `offcut-api`'s `no_media_routes.rs` `every_post_route_refuses_a_body_over_the_limit` failed once in a full parallel workspace run and passed alone and on the re-run: load-sensitive, not a change of this prompt.

**Open.** 35, 36. Next: P188 (TE-15 on DeepSeek, 30 calls, then CP 1).

## Entry 47 - 2026-11-12 - P188: TE-15 on DeepSeek; checkpoint 1 waits for the founder

**No product code changed.** TE-15's full read is in `docs/v6.2/experiments.md`, the 30 annotated plans in `~/v6.2-review/plans/` (outside git) with a README naming the ten to read first. The scratch caller and validating tool were built on the repository's crates and are not committed. A redacted record of one exchange per model is in `~/v6.2-review/records/` for P190's fixtures.

**Every real call (F-9, D-500; the founder cut TE-15 to 30 plans: each model once).** 2026-11-11 23:15:55-23:17:56 UTC, off-peak. Sent: the `@v2` system prompt with the `@v2` schema as text, and the `@v2` user message of `a01` and `w01`-`w14` (written scripts in the repository); never `a02`, audio or a frame. 33 calls, all HTTP 200, all `finish_reason: stop`, no thinking tokens:
- `deepseek-flash`, 17 calls: one per transcript, plus a second for `w03` and `w10` (first answers named emoji outside the 72). Per plan: 4,823-6,989 tokens in (3,840-5,120 cache hits), 460-906 out, 2.4-6.0 s.
- `deepseek-v4-pro`, 16 calls: one per transcript, plus a second for `w05` (its first answer named the icon `cloud`). Per plan: 4,820-6,986 in, 551-1,591 out, 3.1-7.4 s.
- The account's balance went from $1.93 to $1.89: about $0.04 for the 33 calls (off-peak, half the peak prices). Plus one free balance read with no content.

**Against TE-15's lines.** Parse after one retry 15/15 for each model (pass). p90 latency 3.3 s (Flash) and 6.1 s (Pro) (pass). **Median item drop rate 16.7% (Flash) and 28.6% (Pro): both over the 10% line.** Cost per plan at peak: $0.0011 (Flash), $0.0050 (Pro). The terms line ("paid-tier data does not train models") cannot pass for DeepSeek, whose terms say neither; the founder accepted that in D-500 and D-492.

**Why the drops.** Pro anchors every item to its whole sentence, so `AnchorRule` takes 14 of its 30 emphasis marks, 7 of its 12 numbers and most of its icons and emoji. Flash keeps the anchor rules but zooms on almost every sentence (56 zooms in 15 short scripts), losing 12 to the 1.5 s gap and V-3; it also lays icons over numbers (4 `ZoneTaken`). Every parse failure was a single name outside the closed sets, which fails the whole plan under serde.

**Recommendation (open item 37, CP 1).** `deepseek-flash`, thinking off: more items kept, half the latency, a fifth of the cost. Then one prompt revision (one-word anchors written as `{"start": w, "end": w + 1}`; a zoom only where it earns it; no icon over a number) and a 15-call re-read on Flash, about $0.02; and item-level parsing in P190 so an unknown name drops one item, not the plan. Prices to confirm: Flash 0.006 / 0.30 / 1.20, Pro 0.044 / 1.32 / 3.96 USD per 1M tokens (cache hit / miss / output), peak; off-peak half.

**Stop at CP 1, as the prompt says.** P188's acceptance "the founder's choice in the entry" waits for the morning; the choice will be recorded in the commit that applies it. P189 runs tonight, as the founder allowed, with `deepseek-flash` as the configuration's default.

**Counts.** Unchanged (no product code): Rust 939 / 0 / 7, Vitest 650.

**Open.** 35, 36 (done in P189), **37 (CP 1)**. Next: P189.

## Entry 48 - 2026-11-12 - P189: the tables and the configuration

**What changed.** `server/migrations/0002_ai_usage.sql`: `ai_usage`, `ai_shorts_grants`, `ai_spend` exactly as TS §52 (D-462). `db/ai_usage.rs`: `increment` (counts a success, returns the month's count, one upsert), `count`, `month_of` (the first day of the UTC month), and the export's rows. `db/ai_spend.rs`: `add` (adds a call's micro-dollars, returns the month's total), `total`. **S-10:** `USER_TABLES` gains `ai_usage` and `ai_shorts_grants`; deletion takes them by their foreign keys, as before. The export carries `ai_usage` (month, kind, count) and `ai_shorts_grant_months` (month only, never the hash) in a server-side wrapper around `AccountExport`, each left out while empty, so an account that never used the AI plan exports exactly V6.1's JSON and `offcut-api-types` does not change. `server/.sqlx/` gains 6 queries (`cargo sqlx prepare` against the local database, migrated). **Configuration (D-465, D-500):** `config::plan_settings`, read apart from `Config` as `transcribe_budget` is: `PLAN_ENABLED` (default false), `PLAN_PROVIDER` (`deepseek` only), `PLAN_MODEL` (default `deepseek-flash`, the founder's choice for tonight; CP 1 may change it), `PLAN_API_KEY`, `PLAN_API_BASE_URL` (default DeepSeek's), `AI_MONTHLY_CAP_USD`, the two prices. With the plan on, the key, a cap above 0 and both prices are required, and the key is named first; a bad value stops the boot even with the plan off; `Debug` hides the key. `main.rs` reads them at boot (P191 hands them to the route). `render.yaml` lists the eight with `sync: false`.

**Before any other change** (step 1): with the migration alone, today's server tests passed against the migrated schema, 279 of 280; the one failure was S-10's guard (`user_tables_names_every_table_with_a_user_id_column`), which this prompt satisfies.

**Found, and stopped for the founder (open item 38): the V6.1 binary would not boot on the migrated database.** Its `sqlx::migrate!().run()` refuses a database holding a version it does not know (`VersionMissing(2)`; sqlx-core 0.9.0's `validate_applied_migrations`, default `ignore_missing = false`). Reproduced on local Postgres with a scratch program: V6.2's migrations, then V6.1's (0001 only) → `Err(VersionMissing(2))`; the same with `ignore_missing` → `Ok`. §14.4's rollback assumed the opposite. Recommended, not done: `db::migrate` sets `ignore_missing(true)` from V6.2, and the rollback note gains `DELETE FROM _sqlx_migrations WHERE version = 2` before booting V6.1.

**The key (open item 36, the founder's yes).** `.env` and `.env.deploy`: the Gemini `PLAN_API_KEY` line removed, `PLAN_API_KEY2` renamed `PLAN_API_KEY` (names only; no value printed). `.env` (local only) also gains `PLAN_PROVIDER=deepseek`, `PLAN_MODEL=deepseek-flash` and Flash's prices (0.30 in, 1.20 out, per 1M). DeepSeek's balance endpoint answered the renamed key with 200 (no content sent). The gate's comment now names `PLAN_API_KEY` as DeepSeek's.

**DECLARED CHANGE (D-440 (6)).** `account_delete.rs`' `rows_for` inserts an `ai_usage` row and a grant for the person, so its every-table checks cover them; `account_export.rs`' every-table test gains the two tables' arms and their rows (made through `db::ai_usage`), plus a new case: the AI rows by month, the grant without its hash, and an account without AI rows exporting only V6.1's four keys.

**Tests.** New: 5 in `config.rs` (off by default; on reads every value; on without the key, cap or a price stops the boot naming it; a bad value stops it with the plan off; `Debug` hides the key), 1 in `db/ai_usage.rs`, 1 in `account_export.rs`, 2 in `ai_cap.rs` (the month's spend added and read; 20 concurrent adds lose nothing).

**Counts.** Rust **948 / 0 / 7**; `cargo test -p offcut-api` 289; Vitest **650**; `pnpm e2e` **57** (the night's one run, decision 8A, on a fresh `pnpm build`).

**Open.** 35, 37 (CP 1), **38 (the rollback)**; 36 closed. Next: P190, after CP 1.

## Entry 49 - 2026-11-12 - P188b: checkpoint 1, the prompt revision, TE-15 on Flash again

**Checkpoint 1 (open item 37, closed).** The founder took all four recommendations: `deepseek-flash` with thinking off; the budget prices 0.30 in / 1.20 out USD per 1M tokens (as `.env` has had since P189); one prompt revision and a 15-call Flash re-read; item-level parsing in P190. Recorded in D-498. P188's acceptance ("the founder's choice in the entry") is met here.

**What changed.** `offcut-plan/src/prompt.rs`: the finished-short and zoom paragraphs ask for fewer zooms, each covering only its claim's words, and state the 1.5 s gap between zooms; two new paragraphs, on pointing at words (a one-word item anchored `{"start": w, "end": w + 1}`; a number's range exactly the number and its unit) and on the event zone the four event kinds share (1.5 s hold). Their numbers come from the catalog and `validate` (`EVENT_HOLD_MS` is now `pub(crate)` for it). `catalog.rs`: `punch_in`'s purpose says `push` covers "the few words that build to it". The prompt's unit test pins the new lines. **Snapshot changed:** `catalog_snapshot__system_prompt@v2.snap`, read and accepted (it is P186's own; nothing ships on `@v2` yet). The `@v2` output schema and user message are unchanged.

**Every real call (D-500's request).** 2026-11-12 07:16-07:17 UTC, peak hours. Sent: the revised `@v2` system prompt with the `@v2` schema as text, and the `@v2` user message of `a01` and `w01`-`w14`; never `a02`. 15 calls to `deepseek-flash`, all HTTP 200 and `finish_reason: stop`, no thinking tokens, no retry. Per plan: 2.3-3.6 s, median 5,408 tokens in (4,096 cache hits) and 500 out. About $0.017 at peak prices; the balance, shown to the cent, read $1.87 before and after (two free balance reads).

**Result.** Parse 15/15 first try (P188: 13/15). Zoom drops 12 → 1, anchor drops 6 → 0. Median drops 16.7% → **14.3%**, mean 17.8% → 10.9%: **the 10% line still fails on the median.** Seven of the eight number drops left take one word past the number (`$12 a`, `$60 an`, `3 sets of`, `10 pm`). Not acted on, as only one revision was approved: a second line on where a number's range ends, or a trailing-word trim in `validate`. The table and the drops are in `experiments.md`; the 15 annotated plans in `~/v6.2-review/p188b/plans/` (outside git).

**Counts.** `cargo test -p offcut-plan --all-features` green (75). Rust and Vitest otherwise unchanged: 948 / 0 / 7, 650.

**Open.** 35, **38 (the rollback)**; 37 closed. Next: P190.

## Entry 50 - 2026-11-12 - P190: the provider and the DeepSeek adapter

**What changed.** `server/src/plan/` (new, `pub mod plan` in `lib.rs`):
- `provider.rs`: `PlanProvider` (`enabled`, `complete`), `ProviderRequest { system, user, schema }`, `ProviderOutput { text, usage, model }` with `Usage { input_tokens, cached_input_tokens, output_tokens }`, `PlanProviderError` (`Unavailable`, `Timeout`, `Blocked(Usage)`), and `UnconfiguredPlanProvider`. No error holds provider text; `ProviderOutput`'s `Debug` shows the text's length only.
- `deepseek.rs`, the only file that names the provider: D-500's request exactly (the system prompt, then `SCHEMA_LEAD` and the compact schema; `thinking` disabled, `json_object`, temperature 0.2, `max_tokens` 8000; bearer key). `PLAN_UPSTREAM_TIMEOUT` = 15 s (D-498). Any non-2xx (429 never retried, D-459), a body that is not the answer, no choice, or no usage → `Unavailable`; a timeout → `Timeout`; a `finish_reason` other than `stop` or `length` → `Blocked` with its usage, since it was billed. A blank or cut answer returns as text for the parse to refuse. Logs carry `kind` and `status` only.
- `parse.rs`, **checkpoint 1's item-level parsing**: `parse_plan(text, source)` sets `format`, `catalog` and `source` itself (D-457, D-460), reads the plan outside `items` strictly, then each item alone. An unreadable item is left out and counted; the rest stands. A blank, non-JSON or non-plan answer is a `ParseFailure`: its `retry_note()` gives the parser's message to the model for the one retry (D-460); its `Debug` and `Display` show the kind alone.

`server/Cargo.toml` gains `offcut-plan` with `prompt` (D-458). `check-file-tree.mjs` allows the edge. `cargo deny check bans` is ok: `offcut-plan` is already a wrapper of the `offcut-text` ban. TS §5 gains the rows for `deepseek.rs`, `parse.rs`, `deepseek_adapter.rs` and `fixtures/deepseek/`; `provider.rs`'s row is corrected. TS §7 notes the new edge.

**Fixtures (`server/tests/fixtures/deepseek/`, no key or account identifier).** All are real; `id`, `system_fingerprint` and the request id are redacted:
- two answers on `a01`: Flash from P188b and Pro from P188;
- the recorded request, with the texts cut to their lengths;
- the 400 that refused `json_schema` (P183b);
- three first answers from P188 that each named one value outside the 72: the emoji `cat`, `lock` used as an emoji, and the icon `cloud`.

**Tests.**
- `deepseek_adapter.rs`, 7 tests, no real call:
  - both recorded answers read with their exact token counts and parse whole;
  - the three one-bad-name answers fail whole under serde, as in P188, but parse item by item with one item left out each;
  - the request matches the recorded one field for field, and the system message is the same 16,159 characters P188b sent;
  - 400, 401, 402, 429, 500 and 503 all map to `Unavailable`, with one request per call;
  - malformed JSON and no choice map to `Unavailable`, a blocked answer to `Blocked` with its usage, a stall to `Timeout`;
  - a blank answer comes back with its usage, and the parse refuses it as `Blank`;
  - nothing listening maps to `Unavailable`;
  - a captured log holds none of the model's text, the error body, the user message, the prompt or the key.
- 9 unit tests: 2 in `provider.rs`, 3 in `deepseek.rs`, 4 in `parse.rs`.

**Differs from the plan.** The Gemini names read as DeepSeek (D-500). `parse.rs` is new (CP 1). The timeout is D-498's 15 s, not the 25 s that D-500's note repeats from TS §40.7 (recorded in D-500). `ProviderOutput` carries a `Usage` with the cache hits, rather than D-458's two token fields.

**Found.** On Windows, a connection to a closed local port takes about 2 s to be refused. A 2 s timeout races it and reads as `Timeout`, so the "nothing listening" case waits 10 s. `groq_adapter.rs` passes only because it maps both outcomes to one error.

**Counts.** `cargo test -p offcut-api` **305** (289 + 16), with `.env` loaded for `DATABASE_URL`. Rust **964 / 0 / 7** by count (948 + the 16 new): only `offcut-api` changed, and only `offcut-api` and `offcut-plan` were re-run. Vitest 650, unchanged: no web change, not re-run. `clippy -D warnings`, `fmt` and the gate are clean.

**Open.** 35, **38 (the rollback; the founder's yes still due before P212)**. Next: P191 (the route, quotas, the cap; push 1).

## Entry 51 - 2026-11-12 - P189b: the rollback across the migration (open item 38)

**The founder's answer.** Yes to all of open item 38's fix, including the gap found while explaining it: going back to V6.2 after a rollback would rerun `0002` over its own tables. Open item 38 is closed.

**What changed.**
- `db::migrate` sets `ignore_missing(true)`, so this build ignores a version it does not know. A rollback from a later release to V6.2 then boots.
- `0002_ai_usage.sql` creates its three tables `IF NOT EXISTS`. Its checksum changes, which is safe because no deployed database has it yet.
- §14.4 gains the one step for V6.1: `DELETE FROM _sqlx_migrations WHERE version = 2` before V6.1 boots. The tables and their rows stay, unused.
- `migration_rollback.rs` is new, with its TS §5 row.

**Tests (`migration_rollback.rs`, 2).**
1. On a database V6.2 migrated, V6.1's migrator (`0001` alone, as its binary embeds it) is refused with `VersionMissing(2)`. After the step it runs. V6.2's migration then runs again, `ai_spend`'s row is kept, and versions 1 and 2 are recorded.
2. A version 3 recorded by a later release is ignored.

**Found.** sqlx 0.9 returns `VersionMissing` while still holding its advisory lock, on the connection the pool keeps. In production the refused binary exits, which frees the lock. In a test that goes on, the next migration waits forever: the first run hung and was stopped. The test's V6.1 migrator now takes no lock. The hung run left one `offcut_test_*` database behind; it was dropped.

**The local development database** recorded `0002` with its old checksum and would have refused the new file. It was repaired with §14.4's own step: the row deleted, then `sqlx migrate run --ignore-missing`. `0002` ran again over its three tables, with the new checksum.

**Counts.** `cargo test -p offcut-api` 307 (305 + 2). Rust 966 / 0 / 7 by count.

**Open.** 35. Next: P188c (the founder's yes to a second prompt fix), then P191.

## Entry 52 - 2026-11-12 - P188c: where a number's range ends

**The founder's answer.** Yes to the second prompt fix P188b offered (about $0.02).

**What changed.** `offcut-plan/src/prompt.rs`'s pointing paragraph now says where a number's range starts and ends. It starts on the number and ends on the number or on the one unit word just after it (hours, percent, dollars, users), never on `a`, `an`, `of`, `per` or `pm`. Both sides of a from-to are numbers the speaker says. **Snapshot changed:** `catalog_snapshot__system_prompt@v2.snap`, read and accepted; nothing ships on `@v2` yet.

**Every real call (D-500's request).** 2026-11-12 08:03-08:04 UTC, peak hours: 16 calls to `deepseek-flash` on `a01` and `w01`-`w14`, never `a02`. All were HTTP 200 and `stop`, with no thinking tokens. One was a parse retry: `w14`'s first answer named `lock` as an emoji. The calls cost about $0.016 at peak prices; the balance read $1.85 before and after (two free balance reads).

**Result: TE-15 passes on `deepseek-flash`.** Median item drops **0.0%** (mean 8.5%; P188b: 14.3%, 10.9%). Parse after one retry 15/15, p90 3.3 s. No zoom was dropped. Of the 9 drops left, 5 are from-tos whose sides still end on `a` or `an`, 2 are numbers that do not resolve (`3 sets`, `Version 2`), and 2 are icons over numbers. No further revision is proposed. The table is in `experiments.md`; the plans are in `~/v6.2-review/p188c/plans/` (outside git).

**Counts.** `cargo test -p offcut-plan --all-features` green (75). Rust 966 / 0 / 7 by count, unchanged.

**Open.** 35. Next: P191.
## Entry 53 - 2026-11-12 - P191: the route, quotas and the cap

**What changed.**
- `offcut-api-types/src/plan.rs`: `PlanRequest`, `PlanWord` and `PlanResponse` (§7.1), each `deny_unknown_fields`. `ApiErrorCode` gains `plan_quota`, `plan_catalog`, `plan_unavailable` and `plan_invalid`, placed before `internal`. `web/src/gen/api.ts` was regenerated, and `web/src/net/http.ts`' table of codes gains the four (the type requires it).
- `server/src/plan/route.rs`: `POST /api/v1/plan` in §7.2's order (D-460 as read). The `PlanIp` layer and the 64 kB limit come first. The bearer is checked before the body, so a request without one is 401 even when its body is not JSON. Then `PlanUser`, the body, the catalog (409), the switch and the month's spend (502), the person's row (401), the quota (429) and the transcript (400). Then the provider, within 18 s for the whole request, and one parse retry with the parse error appended, made only with 4 s left. The answer is refused over 32 kB, a success is counted, and one log line of counts is written.
- `plan/quota.rs`: Free 40 a month; Creator logs above 1,000 and stops at 3,000; the tier comes from `plan_now`.
- `plan/spend.rs`: `SpendCap` prices each call at the cache-miss price, rounded up, charges blocked answers too, logs one 80% line a month, and refuses at 100%.
- `plan/mod.rs`'s `PlanState` joins `AppState`. `main.rs` builds `DeepseekProvider` when `PLAN_ENABLED=true` and `UnconfiguredPlanProvider` otherwise.
- `rate_limit.rs`: `PlanIp` and `PlanUser`, each 10 a minute.
- `error.rs`: the five plan errors; a timeout is 504 with `plan_unavailable`.
- `db/users.rs`: `exists`, with its query added to `server/.sqlx/`.
- TS §5 rows for `offcut-api-types/src/plan.rs` and `server/tests/common/plan.rs`. `/healthz` is unchanged.

**Tests.**
- `plan_route.rs` (14): the plan as the model wrote it, with the server's source, counted and charged, and the system and user messages checked; the source set whatever the model claimed (catalog 1, the rules); no bearer is 401 before the body, with nothing sent, counted or spent; an unknown field, another catalog and a tab in a word are each refused before the provider; the switch off; a deleted account with a live token; the Free 41st, with last month's 40 not counting; a Creator's 3,001st; one parse retry with the error in the second message, counted once and charged twice; two bad answers are `plan_invalid`, charged and not counted; an unknown emoji leaves its item out with no retry; unavailable is 502, timeout 504, blocked 502 and charged, none retried; over 32 kB is `plan_invalid` and not counted; ten a minute per person across addresses, and ten per address before any bearer.
- `ai_cap.rs` (+1): at the cap the 4th call is refused and not sent; last month's full spend does not close this month.
- `ai_quota.rs` (2, new): only a plan that came back is counted; each month and each person counts alone.
- `log_redaction.rs`: the plan case. A word of the request, the model's text and a parse error quoting it never appear in the log; the line shows `retried` and both calls' tokens; the 80% line appears exactly once.
- `no_media_routes.rs`: `/plan` is in `READ_BODY`, so it is refused over 64 kB and with each media type.
- Unit tests (9): `route.rs` 3 (the transcript checks, the buckets), `quota.rs` 2, `spend.rs` 2 (cost, once a month), `error.rs` 1 (the plan errors beside the table), `rate_limit.rs` 1 (the plan groups beside the table). `offcut-api-types` gains 1 (`plan.rs`).

**DECLARED CHANGES (D-440 as read).** None of these deletes an assertion.
- (7) needed no edit: a test beside the table, as V6.1 did.
- (14) `errors.rs`: 14 codes → 18, and the test's name.
- (15) `lib.rs`' TypeScript test: 43 → 46 names, with four imports added.
- (16) `no_media_routes.rs`' `READ_BODY` gains `/plan`.
- (17) `log_redaction.rs` gains the plan block.

**Found, and fixed here: P188c broke a P190 test.** `deepseek_adapter.rs` pins the system message to the recorded request's length. P188c lengthened the prompt, and P188c was committed after only `offcut-plan`'s tests, so `the_request_is_the_one_recorded` failed from `031a9ec` until this commit. Its recorded request is now P188c's (16,366 characters).

**Found.**
1. A thread-local log subscriber missed the 80% line when its test ran beside others, though it passed alone (tracing's callsite cache). The check moved into `log_redaction.rs`, whose subscriber is the process's.
2. The local `.env` cannot boot the full server: `MAIL_FROM` is not an address, and `BILLING_WEBHOOK_SECRET` lacks `whsec_`. Both are 14-character placeholders. The real call ran with stand-ins for those two, for that process only. `.env` was not changed.
3. §14.2's deploy check expected `E_PLAN_UNAVAILABLE` for a bare request; with D-494 it is 401 first. Corrected in the plan.

**The one real local call (F-9, step 4).**
- 2026-11-12 08:23:31 UTC, peak hours. The real server on `offcut_dev` with `PLAN_ENABLED=true` (that process only) and DeepSeek's key.
- A throwaway user, deleted after (its `ai_usage` row went with it), and an access token signed in-process with the local key, never printed.
- The body was `a01`'s `PlanRequest` (157 words, 17 sentences, 11.6 kB, the repository's script).
- **Result:** HTTP 200 in 3.8 s (provider bucket `lt5`); 9 items: 2 numbers, 3 zooms, 1 from-to, 1 list, 1 icon, 1 emphasis. Source `deepseek` / `deepseek-flash`, catalog 2, `plans_remaining` 39; a hook, a caption and 6 hashtags; counted once.
- The log line: `{"message":"plan","kind":"ok","provider_s_bucket":"lt5","input_tokens":7303,"cached_input_tokens":7168,"output_tokens":668,"retried":false,"items":9,"items_skipped":0}`, inside the request's span (route, hashed user). No word of the transcript appears anywhere in the server's log.
- The balance read $1.83 before and after (one call, about $0.003).

**Counts.** Tier 3's server and Rust parts, run in full: `pnpm test` with `.env` loaded gives **Rust 993 / 0 / 7** (966 + 27: 26 in `offcut-api`, now 333, and 1 in `offcut-api-types`, now 32) and **Vitest 650**. `pnpm check` is clean (clippy on the workspace, cargo-deny, eslint, both `tsc`, the file tree, gen-clean, copy codes). The golden frames pass, 19. Browser e2e and `pnpm fixtures` were not run (no client change but the four codes; P192 runs the client).

**Push 1.** `v6.2-build` pushed after this commit (not waited on, as the prompt says).

**Open.** 35. Known issues 4 and 5 (P192, P212). Next: P192.

## Entry 54 - 2026-11-12 - P192: flags, the request, the store

**What changed.**
- `config/features.ts`: `ai_plan` and `motion`, both default off (D-443). `featureFlags`' one-line return of three flags becomes a return of five, the one removed line Step 0.4 allows; `scripts/v62-gate.sh` gains its allow-list pattern. `vite.config.ts` is unchanged: its define writes whatever `featureFlags` returns.
- `package.json`'s `build:e2e:flags` and `ci.yml`'s `dist-flags/` step: `long_video,high_res,ai_plan,motion` (D-444). Nothing reads the two new flags yet, so `dist-flags/` behaves as before.
- `net/api-client.ts` (additions only): `PLAN_CLIENT_TIMEOUT_MS = 20_000` and `requestPlan(body, signal)`, through `authed` (one refresh on a 401), **one attempt**, 20 s.
- `usecases/request-plan.ts` (new): `planFor({ transcript, prosody, duration, switchOn, signal })` → `{ source: "model", plan, plansRemaining }` or `{ source: "rules", why }` (D-466).
  - `ai_plan` off or the switch off → `off`; not signed in → `signed_out`; offline → `offline`. Each at once, with no request.
  - Otherwise one `POST /plan` and a 20 s timer over the whole request; the request is aborted when the timer wins.
  - Codes map as D-466's note says. It never throws.
  - `planRequestOf` builds the body from the transcript as heard and the prosody: four fields, nothing the person typed.
  - No caller yet (P193).
- `persistence/schema.ts` (additions): `PLANS_STRUCTURE_VERSION = 3`, `PLAN_STORES`, `PlanRecord { plan, source: "model" | "rules", requestedAt, catalog }`, `OffcutPlansDb`. `DB_STRUCTURE_VERSION` stays 2.
- `persistence/db.ts` (additions): upgrade 3 makes `plans`; `openPlansDb()` copies `openSourcesDb`: asks for 3, opens a newer database as it is, closes itself when another tab upgrades.
- `persistence/plans-repo.ts` (new): `put`, `get` (a newer `schemaVersion` is absent and removed), `clear`; failures are `OpfsError`.
- TS §5: rows for the four new files; `features.ts`' row says five flags; `plans-repo.ts`' row loses "its op log" (D-469 has none).

**Known issue 4 (closed).** `requestPlan` uses `retry: "none"`, as `transcribeAudio` does. A 502 or 504 `plan_unavailable` and a 502 `plan_invalid` come back on the first answer with their code, so `planFor` reads them as `unavailable` and `invalid` at once. A proxy's 502 to 504 with no body (an API still waking) is one attempt too, read as `unavailable`: the plan never waits for a cold start, and the clip goes on with the rules plan. The other fix (two new `ErrorCode`s in `PROVIDER_FAILURES`) was not taken: it would change `offcut-types`, the generated files and `http.ts` for no gain. A check: with `retry: "cold-start"` put back for a moment, 5 of the new cases failed.

**Tests.**
- `request-plan.test.ts` (29, new):
  - off by the flag, off by the switch, four signed-out states, offline: each with no request;
  - the model's plan, and a Creator's without a count;
  - 12 failures, each to its outcome;
  - a 200 without a plan is `invalid`; a client that throws, at once or later, is `unavailable`;
  - a caller's cancel reaches the request;
  - no answer by 20 s is `timeout`, with the request aborted (not at 19.999 s); an answer at 19.999 s is used and no timer is left;
  - the body field for field; word edits in the clip store never reach it; a missing prosody entry reads as 0.
- `plans-repo.test.ts` (9, new): the real `db.ts` and `plans-repo.ts` over an in-memory `openDB` that keeps IndexedDB's version rules (no npm package added, §3.1):
  - **1 → 3 and 2 → 3 keep every record**, compared store by store, and the clip path and long path read theirs after;
  - **the clip path still opens version 1 and the long path 2**, neither making `plans`;
  - a new page's clip path on a version-3 database asks for 1, is refused and opens it as it is; a database at 4 opens as it is;
  - records, the newer-`schemaVersion` rule, the sweep, `OpfsError` quota and io;
  - only `db.ts` and `plans-repo.ts` name `openPlansDb` (D-469).
  - Two changes to `db.ts`, each tried for a moment, failed 7 and 4 of these.
- `api-client.test.ts` (+7, new cases only): the body once as JSON with the token and one refresh; 502 `plan_unavailable`, 504 `plan_unavailable` and 502 `plan_invalid` each one attempt; a proxy's 503 one attempt; a 401 after the refresh is `E_AUTH_SESSION_EXPIRED` with no third attempt; the attempt may take 20 s.
- `features.test.ts`: one new case (the four names on, each new flag alone, a name in both lists).

**Real browser (once, not committed).** The Vite dev server and installed Chrome, through the real `db.ts` and `plans-repo.ts`:
- 1 → 3: 8 stores → 12, version 3, with the clip, transcript, edits and `meta` records read back;
- 2 → 3: 11 → 12, with the source and window records read back too;
- in both, a reloaded page's clip path read the version-3 database.

**DECLARED CHANGE (D-440 (18), as read).** `features.test.ts`' two cases that compare the whole flag object gain `ai_plan: false, motion: false`; every old key and value is kept, and no assertion is deleted. `api-client.test.ts` gains new cases only.

**Differs from the plan.**
- The timer is 20 s (D-466, D-498), not §8.1's or TS §40.7's 30 s; TS's flow line still says 30 s, for P212's copy-back.
- `ci.yml` changed as well as `package.json`, since CI builds `dist-flags/` with its own list.
- `planFor` takes the switch as an input (P193 reads it).

**Found.**
1. Chrome's headless shell is not installed for Playwright here; the check used installed Chrome (`channel: "chrome"`), as the e2e suites do.
2. Opening 1 → 3 also makes the three source stores, empty, in a build with `long_video` off (D-469's note).
3. Two things for P193, as known issue 6: the clip sweep must clear `plans`, and a caller's cancel must not be reported.

**Counts.** Vitest **696** (650 + 46: 29 + 9 + 7 + 1). Rust 993 / 0 / 7 by count, unchanged: no crate or server file changed (`pnpm check`'s clippy covered the workspace). `pnpm check` is clean (clippy, cargo-deny, eslint, both `tsc`, the file tree, gen-clean, copy codes). `pnpm build`'s bundle has `cloud_transcribe:!0,long_video:!1,high_res:!1,ai_plan:!1,motion:!1` (in the page's `index-*.js`). The gate prints `gate: ok`. Browser e2e was not run: no page reads the new flags or calls the new code yet (P193, P194).

**Open.** 35. Known issues 5 and 6 (P212, P193). Not pushed (push 2 is P197). Next: P193 (the pipeline chooses the plan).

## Entry 55 - 2026-11-12 - P193: the pipeline chooses the plan

**First, before any change.** The call-sequence case in `run-pipeline.test.ts`: with `ai_plan` off, the run's calls are exactly today's 15 (`DEVICE_CALLS`), in order, with no `plan_result`, a person signed in and the card on this device. It passed on the unchanged code (30 of 30). After the change it still passes, and the mocks of `request-plan` and `plans-repo` write into the same call list, so any plan call with the flag off would fail it.

**What changed.**
- `usecases/choose-plan.ts` (new):
  - `startPlan` (step 11b): with the switch on, a plan kept for this clip is used with no request; otherwise `planFor`, with whether a request went out and how long it took.
  - `choosePlan` (step 12b):
    - a model's plan that keeps at least one item is used; else the rules plan, with the reason;
    - a plan asked for now is kept for the clip, the rules plan too when the model's could not be used;
    - a fallback other than `off` is set on the clip once (`planFallback`);
    - one `plan_result`.
- `run-pipeline.ts`: 11b starts the plan only with `FEATURES.ai_plan`; 12b waits for it inside `detect_scene`, after the rules plan, then `setScene` once with the plan chosen. One line changed (the rules plan's name), 576 lines.
- The binding: `validate_plan` in `scene_api.rs` (additive): `validate` alone, no session, `{ kept, dropped }` or `{ refused }`. It is wired through `load-render.ts`, the render worker's `validatePlan` and `protocol.ts`' `PlanCheck` (additive).
- `review-update.ts`:
  - a word edit keeps a model's plan, and the scene validates it against the edited words; only a rules plan is made again;
  - `replacePlan` puts a plan in place of the clip's in the update queue.
- `regenerate-plan.ts` (new): asks again, checks, replaces, keeps; any failure changes nothing and returns the reason.
- The settings switch:
  - `META_KEYS.aiPlan` (the gate's allow list gains the one-line `META_KEYS`);
  - `prefs-repo.ts`' `getAiPlanSwitch` and `setAiPlanSwitch` (on unless stored `false`);
  - `state/plan-store.ts` and `usecases/set-ai-plan.ts` (new);
  - `start-app.ts` reads the switch only with the flag on.
- `import-clip.ts`: the sweep clears `plans`, only with the flag on (known issue 6a).
- `clip-store.ts`: `planFallback` and `planFellBack`, as `asrFallback`.
- `plan_result` (D-490):
  - `offcut-api-types` gains `PlanResultSource`, `PlanOutcome`, `LatencyBucket`, `MAX_DROPPED = 99`, the variant and its description;
  - the server bounds `items` (10,000) and each drop count (99);
  - `messages.ts` gains its name;
  - `pnpm gen:types` was run.
- `request-plan.ts`: its three "no request" checks are now `reasonNotToAsk`, exported so the pipeline knows a request went out; `planFor`'s behaviour is unchanged (its 29 tests pass).
- `eslint.config.js`: the edges `run-pipeline → choose-plan`, `choose-plan, regenerate-plan → request-plan`, `regenerate-plan → review-update`.
- TS §5: rows for the six new files.

**Tests.**
- `run-pipeline.test.ts` (+19, new cases only):
  - the call-sequence case;
  - a model plan chosen: asked right after the prosody, checked before the scene, kept, `plan_result` exact;
  - a model plan with some drops still wins, the drops counted by reason;
  - **a fully dropped model plan**: the rules plan as `invalid`, kept, its drops counted;
  - a newer catalog: `catalog`;
  - **each fallback** (7): quota, unavailable, invalid, timeout, offline, catalog, signed out (signed out keeps nothing and has no latency);
  - the switch off: no stored read, nothing kept, no line;
  - **restore makes no request** (3): a kept model plan used and checked with no request and no second `plan_result`; a kept rules plan, no request and no check; a kept plan that no longer keeps an item gives the rules plan, no request;
  - Retry after a failure past the plan asks nothing;
  - a cancel while the plan is awaited ends `accepted`, with no scene, nothing kept and nothing reported;
  - a word edit on a model's plan keeps it (`redetectSentence`, `setScene`, `editsRepo.put`; no `rulesPlan`).
- `regenerate-plan.test.ts` (16, new): success with the edit kept whole and no history step; 8 request outcomes; every item dropped, newer catalog, another format; a scene that fails; a worker that cannot check; a clip that went; not ready; flag off.
- `set-ai-plan.test.ts` (4, new): on by default and when unreadable; kept under `aiPlan`; a failed write.
- `scene_api.rs` (+2): kept and drops reported without a session; refused whole; a bad word-edit key.
- `analytics.rs` (+1): a fallback without latency; a map key or value off the list is refused.
- `analytics_allowlist.rs` (+1): stored as sent; 100 drops and 10,001 items refused; at the caps accepted.
- Breaking the code on purpose:
  - each of four changes failed the new cases: a fully dropped plan used (1), no stored read (4), the flag ignored (13, the call-sequence case among them), a word edit making the rules plan (1);
  - removing `regeneratePlan`'s empty-plan guard failed 1.

**DECLARED CHANGES (D-440, as read; none deletes an assertion).**
- (5) `analytics.rs`: 20 → 21 samples, 17 → 20 enum rows.
- (15) `lib.rs`: 46 → 49 names, `DropReason` imported, 21 docs.
- (19) `server/tests/analytics_allowlist.rs`: `one_of_each` gains `plan_result`, 20 → 21 rows, 22 → 23 bodies.

**Differs from the plan.**
- `PipelineStage::Plan` is not added (D-467's note: it widens the web's failure stages; `latency_bucket` measures the wait).
- The plan helpers live in `choose-plan.ts`, not `run-pipeline.ts` (700 lines).
- §8.3's "`restore-session.ts` reads `plansRepo`": that file restores the account; there is no clip restore before V7, so the stored plan is read when a clip's run starts again.
- The interface is P194's (known issue 7).
- No `plan_result` for a reused plan or for "Regenerate plan".

**Found.**
1. `review-update.ts` would have replaced an AI plan with the rules plan on the first word edit, while the store kept the AI one. Fixed here (D-469's note).
2. `run-pipeline.ts` would have reached 709 lines with the helpers inside it.
3. `eslint --fix` removed `as` casts in two `vi.hoisted` objects that gave their fields their types; `tsc` caught it, and the objects now have return types.

**Counts.**
- Vitest **735** (696 + 39: 19 + 16 + 4).
- Rust **997 / 0 / 7** by count (993 + 4): `offcut-wasm-render` 11, `offcut-api-types` 33, and `offcut-api` 334 run in full with `.env` loaded.
- `pnpm check` is clean (fmt, clippy on the workspace, cargo-deny, eslint, both `tsc`, the file tree, gen-clean, copy codes). The gate prints `gate: ok`.
- `landing.spec.ts` on a fresh `pnpm build:e2e`: 30 passed. Its table test counts the events against the generated list, so it checked the new `plan_result` row.
- No media suite was run: with the flag off the run's calls are unchanged.

**Open.** 35 (the privacy page and settings must name DeepSeek before `ai_plan` goes on). Known issues 5 and 7 (P212, P194). Not pushed (push 2 is P197). Next: P194.

## Entry 56 - 2026-11-12 - P194: the list, the text extras, the copy

**What changed.**
- **Copy:**
  - `copy/plan.ts` gains `planCopy` with §12's drafts, DeepSeek in place of Gemini (D-500);
  - `fallbackLine` and `regenerateLine` choose the line for a reason;
  - the Enhance and Glow drafts wait there for P201 and P208;
  - `messages.ts` gains the import and one key, `plan` (683 lines).
- **The visuals list** (`PlanItemList.tsx`):
  - the five kinds get their own Lucide 1.61.0 icons in `icons.tsx` (highlighter, shapes, zoom-in, smile, heading; with copy and refresh-cw for the buttons);
  - "Regenerate plan" sits under the list, also under an empty one, with `ai_plan` and the switch on. It is disabled while an export runs or a change is applied. A plan that changes nothing gives one toast with the reason.
- **`TextExtrasPanel.tsx`** (new): "Text for your post" under the visuals. The hook, the post caption and the hashtags (as one line) each have Copy, which turns to "Copied" for 2 s and is announced to a screen reader. It shows only extras that passed, and nothing on a rules plan.
- **The extras that passed reach the page.** `validate_plan` answers `extras` (D-454's note). The clip store holds `planExtras`, set where a model's plan is used: the pipeline, a stored plan, regenerate. A run again starts without them.
- **The fallback toast:** `usePlanFallbackToast` beside the cloud's, in `Stage`, once a run. Signed out, it says that signing in gives an AI-planned edit (D-494).
- **The settings page, with the flag on only:**
  - the "AI edit plan" switch and its line;
  - a row of "What leaves your device" after the sound's (open item 35's wording, a draft for CP 3).
  - With the flag off the page and its table are as before.
- **Dev hooks:** `flags`, `fakePlan` (a model's plan with one item of each new kind and its extras, on the list only, the scene not rebuilt) and `fakePlanFallback`.
- **`screens.spec.ts`:** one new test, skipped unless the build has `ai_plan`.
- **TS §5:** rows for `TextExtrasPanel.tsx` and `copy/plan.test.ts`; `PlanItemList.tsx`' row.

**Captures** (`OFFCUT_FEATURES_ON=ai_plan pnpm screens --grep "V6.2"`, development build, D1). 14 files in `~/v6.2-review/ui/`, each at 1280 x 800 and 1920 x 1080:
- `plan-visuals`: the list with the five kinds, the signed-out toast of the real run;
- `plan-text-extras`;
- `plan-copied`;
- `plan-fallback-quota`;
- `plan-regenerate-signed-out`;
- `settings-ai-plan-on`, `settings-ai-plan-off`.

The real run, signed out, sent no request and showed the sign-in line. Copy put the hook on the clipboard (read back). The switch off was kept across a reload.

**axe.** No serious or critical violation (the suites' `axe` helper) on each of the 7 states, at both widths.

**Tests.**
- Vitest (+9):
  - `run-pipeline.test.ts` +5: a model plan's extras reach the clip; none when every item is dropped, none after a failed request; a stored plan brings them back with no request; a run again starts without them;
  - `regenerate-plan.test.ts` +1: new extras replace the old, and a failure keeps them;
  - `copy/plan.test.ts` (3, new): each fallback's and regeneration's line; the settings line names DeepSeek and China and promises no keeping or training.
- `scene_api.rs` +1: the extras that passed, a bad hashtag left out.
- Removing the pipeline's `setPlanExtras` for a moment failed 1.
- `review-edit.spec.ts`, unchanged: 18 passed on a fresh keyed `pnpm build:e2e` (`dist/`, `ai_plan:!1`).
- `landing.spec.ts`: 30 passed on the same build (the settings table, flag off).

**Declared.**
- P193's two `validate_plan` tests in `scene_api.rs` (V6.2's own, not of the base) gain the `extras` field in their expected JSON; nothing removed.
- No test of the base changed.
- `screens.spec.ts` gains one test; its existing tests are unchanged.

**Differs from the plan.**
- §12's Gemini line reads DeepSeek.
- Three regeneration lines and the settings row are new drafts (§12's as-read note).
- `plan.processing` is not shown (known issue 8).
- The captures need a flag-on development build, as the fake API has no `/plan` until P210; the model's plan is put on the list by a dev hook.

**Found.**
1. The page had no way to know which text extras passed. Fixed through `validate_plan`.
2. `pnpm screens` writes its development build into `dist/`, so `dist/` must be built again (`pnpm build:e2e`) before a media suite.

**Counts.**
- Vitest **744** (735 + 9).
- Rust **998 / 0 / 7** by count (997 + 1; `offcut-wasm-render` 12).
- `pnpm check` is clean; the gate prints `gate: ok`.

**Open.** 35: the settings line and row are drafted; the privacy policy's own text still needs DeepSeek before `ai_plan` goes on, and its wording is the founder's at CP 3. Known issues 5 and 8 (P212, P210). Not pushed (push 2 is P197). Next: P195 (the images; checkpoint 2b).

## Entry 57 - 2026-11-12 - P195: the image bundle

**Founder's decisions at CP 2b (2026-11-12).**
- **The 72 images are approved as drawn, with no swaps** ("I don't want to swap anything until anything is breaking").
- **160 px, with D-471's size line raised from 600 kB to about 1.0 MB:** "effortless user experience should be prioritized; most customers use 5G and wifi; downloading a few extra MBs might not be as big a deal".
- The sheets they read: `~/v6.2-review/images.png` (all 72 at 1×, emoji on mid-grey, icons on mid-grey and on dark, then all at 48 px), `images-2x.png`, `images-128-vs-160.png`, `images-README.md`.

**What changed.**
- **`scripts/gen-image-bundle.mjs`** (new; `node scripts/gen-image-bundle.mjs`, with `--out` and `--size` for dry runs):
  - reads the names and upstream files from `assets.rs`' variant docs;
  - fetches each at its pin: Fluent Emoji `raw.githubusercontent.com/microsoft/fluentui-emoji/1ffb34c…/assets/<Name>/3D/<name>_3d.png` (the six hand gestures from `Default/`), and `unpkg.com/lucide-static@1.61.0/icons/<name>.svg`;
  - emoji: resized 256 → 160 px in Node, Lanczos-3 on premultiplied colour, written with straight alpha;
  - icons: drawn white by the installed Chrome (Playwright, `channel: "chrome"`, GPU off, network blocked) on Lucide's 24-unit grid, stroke 2;
  - encodes every PNG itself (IHDR, IDAT, IEND only), keeping the smallest of a few filter and zlib settings;
  - downloads go to a fresh temporary folder.
- **`crates/offcut-scene/assets/images/`** (new):
  - `emoji/` (48) and `icons/` (24): 160 × 160 RGBA PNGs, 997,875 bytes;
  - `manifest.json`: per image its set, id, code, key, file, bytes, SHA-256, pinned source, the upstream file's SHA-256 and the licence. A later run fails if a pinned upstream file changed;
  - `LICENSES/fluent-emoji-MIT.txt` and `LICENSES/lucide-ISC.txt` (with Feather's MIT), as shipped.
- **`offcut-scene/src/images.rs`** (new):
  - `ImageSet::embedded()` (`include_bytes!` of each PNG by `ImageKey`, `Clone`), with `get`, `contains`, `iter`, `len` and `SIZE = 160`;
  - `ImageKey::of_emoji` (the code) and `ImageKey::of_icon` (`ICON_BASE` 1000 + the code);
  - `lib.rs` exports it.
- **`web/LICENSES.md`:** two rows for the bundle and Fluent Emoji's MIT text. The bundle's Lucide icons are covered by the Lucide section; eight of them are Feather's.
- **`offcut-scene/Cargo.toml`:** `png` and `sha2` as dev-dependencies only. Both are workspace dependencies already, so `Cargo.lock` gains two lines in the crate's list and no package. `cargo deny check` is clean.
- **Docs:** TS §5 rows (the script, `images.rs`, `assets/images/`, `image_bundle.rs`); "As read (P195)" notes under D-470, D-471 and §3.1.

**Determinism.** Two runs into scratch folders, then a third in place: byte-identical files, icons included (Node 24.19.0, Chrome, D1).

**The render binary (`offcut_render_bg.wasm`, release, `pnpm build:wasm`).**

| Build | Bytes | Growth | Brotli |
|---|---|---|---|
| Before (`67d6247`) | 6,481,347 | | 2,452,729 |
| After, as committed | 6,481,347 | 0 | |
| With every image kept (a temporary reference, not committed): what P196/P197 ship | 7,480,690 | **+999,343** | 3,451,389 |
| The same at 128 px (D-471's fallback, not taken) | 7,220,049 | +738,702 | 3,191,793 |

Nothing in `offcut-wasm-render` uses `ImageSet` until P197, so the linker drops the images today. P183's estimate (10-11 kB an emoji, from Fire) was low: they average 19.7 kB.

**Tests (+7).**
- `images.rs` +3: the keys are the codes and the icons start at the base; one image per name, in key order, the same bytes when asked again; `Debug` shows lengths.
- `tests/image_bundle.rs` (new, 4):
  - the manifest's names equal `EmojiId::NAMES` and `IconId::NAMES` in order (72 in all), with each code and key;
  - each file is a 160 × 160 RGBA PNG, not blank, with its SHA-256;
  - keys and files are distinct, and the embedded bytes of each key hash to the manifest's;
  - each source is pinned and each licence file exists.
- Swapping two names in `images.rs` for a moment failed 1.
- Re-run: `offcut-types` 93 / 0 / 1, `offcut-scene` 102 / 0 / 4, `offcut-render` 25 / 0 / 0 (`golden_frames` 19 passed, no golden touched), `offcut-wasm-render` 12.

**Declared.** No existing test changed. No web code changed, so Vitest was not run.

**Found.**
1. Every transparent pixel is 0, 0, 0, 0. P183 read no dark fringe with straight alpha; P196's `image_basic` reads it on these files (known issue 9).
2. `ImageSet` is not in `SceneInput` yet. P204 adds it where the scene first emits `Image` (§9.2).

**Counts.** Rust **1005 / 0 / 7** by count (998 + 7); Vitest **744**. `pnpm check` is clean; the gate prints `gate: ok`.

**Open.** 35. Known issues 5, 8 and 9 (P212, P210, P196). Not pushed (push 2 is P197). Next: P196 (the renderer draws images).

## Entry 58 - 2026-11-12 - P196: the renderer draws images

**P195 checked again first.** The script, run into a fresh folder, reproduced the committed bundle byte for byte. Its 7 tests pass. The commit holds its 86 files and nothing else.

**What changed.**
- **`vello_backend.rs`:**
  - `decode` reads a bundle PNG into peniko's `ImageData` (`Rgba8`, straight alpha, as the file holds it).
  - `Overlay::set_images` decodes each file once and replaces the set held before; on an error the old set is kept.
  - The overlay's `Image` arm draws with `draw_image`: bilinear (`ImageQuality::Medium`), the command's opacity, scaled into its rect, in list order with the text.
  - A key not held, or a rect or opacity that is not a number, is `Unsupported`, never a blank (§9.2). An empty rect or an opacity of 0 draws nothing.
- **`lib.rs`:** `Renderer::set_images(&ImageSet)`; `render`'s signature is unchanged; `Unsupported`'s doc names the image cases.
- **`composite.rs`:** `video_of` accepts `Image` after the video (D-473, DC-8's V6.2 part). `Compositor::draw` and `video_pass.rs` are untouched.
- **`Cargo.toml`:** `png` becomes a normal dependency of `offcut-render` (D-472, §3.1), and its dev-dependency line goes. No package is added; `cargo deny check` is clean.
- **Docs:**
  - TS §5 rows (`vello_backend.rs`, `image_draw.rs`, `fixtures/golden/`);
  - "As read (P196)" under D-472 and D-473;
  - TE-30's P196 read in `experiments.md`.

**Differs from the plan.** Nothing is premultiplied at decode (the prompt and §9.2 say "premultiply"). D-472's P183 note holds: Vello 0.11's shader premultiplies each texel before it filters, and an 8-bit premultiply would lose colour at soft edges. The pixel test proves the result, and marking the pixels premultiplied fails it by 127 levels.

**Tests (+7).**
- `vello_backend.rs` +3: a bundle file decodes to its straight-alpha pixels; a file that is not an RGBA PNG is refused; an image is scaled into its rect, then moved.
- `tests/image_draw.rs` (new, 3, WARP):
  - every pixel of an emoji at 1x is its file blended over grey (worst 0.50 of a level), and at opacity 0.5 (worst 0.74);
  - a white icon at 0.5x, 1.37x and 1.5x between pixels has soft edges and no dark fringe (known issue 9, closed);
  - a key not held is `Unsupported`: before any set, between the sets, past the icons, after an empty set. A NaN rect is `Unsupported`; an empty rect draws nothing.
- `golden_frames.rs` +1, **`image_basic`**:
  - three emoji at 0.5x, 1x and 1.5x (the last between pixels, over the disc), one at opacity 0.5, one across the right edge, a white icon, and a half-dark rectangle over one emoji (list order);
  - written once on WARP, read at 200%: clean edges, no rim, the cut at the edge, the rectangle over the emoji.

**Declared.**
- D-440 (3): in `composite.rs`' `a_list_outside_v4s_rule_is_unsupported`, the case "an image" (`[video, image]`, `Unsupported`) is replaced by "an image before the video" and "an image and no video", both still `Unsupported`. `a_scenes_list_is_drawn_with_its_video` gains `[video, text, image, pop]`, drawn.
- `golden_frames.rs` gains one line in `golden()`, so every golden is drawn with the bundle held, plus two `use` lines and the new test. No line is removed.

**Every old frame unchanged.** All 16 old goldens read SSIM **1.0000** with the images held; parity 0.9996, 0.9997, 0.9998. Rewriting every golden to compare bytes:
- 14 come out byte-identical;
- `tech_list` has the same pixels with different PNG bytes (an older encoder);
- `bold_number` differs in 0 to 2 pixels by 1 level from run to run. It does so **with and without P196** over five draws, so it is WARP's own variation, which the SSIM line allows.

All restored; no golden of the base is modified.

**Render binary.** 6,481,347 → **6,484,358** bytes (+3,011): the image arm makes Vello's image path reachable. The decoder and the 1.0 MB of images arrive with P197's `set_images` call.

**Counts.**
- Rust **1012 / 0 / 7** by count (1005 + 7). `offcut-render` 32 (9 + 3 + 20); `offcut-wasm-render` 12 and `offcut-scene` 102 / 0 / 4, both re-run.
- Vitest **744** (no web code).
- `pnpm check` is clean; the gate prints `gate: ok`.

**Open.** 35. Known issues 5 and 8 (P212, P210); 9 closed. Not pushed (push 2 is P197, waited on: CI's lavapipe must pass `image_basic`). Next: P197 (images in the browser, TE-30).

## Entry 59 - 2026-11-12 - P197: images in the browser, TE-30; push 2

**What changed.**
- **`offcut-render/src/check.rs`** (new, `pub mod check`): TE-30's frame defined once, the goldens' synthetic source (`pattern`) and `image_basic`'s list. `golden_frames.rs` asserts both equal its own:
  - `image_basic` gains `assert_eq!(list, check::image_basic())`;
  - a new test compares the two patterns at 1080 × 1920 and 1920 × 1080, and the canvas size.
- **The binding:**
  - `session.rs` (additive only): `attach_canvas` hands `ImageSet::embedded()` to the renderer when it makes it; a later canvas keeps them. `render_error_value` is added for the check.
  - `check_api.rs` (new): `check_source()` and `draw_image_check(canvas, frame)`. The second uses a renderer of its own with the images held, kept alive until the page has read the canvas, and gives `decode_ms()`.
- **The worker passes nothing** (the prompt says it passes the images). The images are in the WASM (D-471), so the binding hands them over itself ("As read (P197)" under D-474).
- **Web:**
  - `protocol.ts` (additive): `devImageCheck` and `ImageCheckResult`;
  - `render.worker.ts` serves it only when `DEV_FAULTS` holds;
  - `workers/render/image-check.ts` (new): a `VideoFrame` of the source (`RGBA`), the draw, and the canvas read back through an `ImageBitmap`;
  - `load-render.ts`: `checkSource`, `drawImageCheck`;
  - `usecases/dev-faults.ts`: `imageCheck`;
  - `ui/dev-hooks.ts`: `__offcutDev.imageCheck()`.
- **`tests-e2e/image-parity.spec.ts`** (new, in `MEDIA_SUITES`; no `@no-gpu` case, so D1 only) and **`helpers/ssim.ts`** (new): a line-for-line copy of `ssim.rs`, which gives Rust's numbers to 10 decimals on three golden pairs. The browser's frame is kept in `web/test-results/image-parity/`.
- **Docs:** TS §5 rows (`check.rs`, `check_api.rs`, `image-check.ts`, `helpers/ssim.ts`, `image-parity.spec.ts`); "As read (P197)" under D-474; TE-30 in `experiments.md`.

**TE-30 (D-474): passes on D1.**
- **SSIM 0.9995** (line 0.98), browser against the WARP golden, on Chrome's WebGPU, Intel "gen-12lp".
- Read by eye at 200%: the same placement, edges, opacity, order and cut as the golden.
- **The 72 images decode in 24 ms**, once per renderer, in the worker. `pipeline-preview`'s "no main-thread task over 100 ms from the drop to ready" passes.

**The render binary.** 6,484,358 → **7,496,980** bytes: +1,012,622 for this prompt, **+1,015,633 against P195's base** (the images, the PNG decoder and the check), the ~1.0 MB the founder approved at CP 2b.

**Production build** (checked in a scratch folder):
- no `__offcutDev`, no `imageCheck`, and none of `image-check.ts`' text;
- what stays is the `devImageCheck` stub, which refuses (`NoHandler`), and the two loader wrappers (about 100 bytes);
- the WASM carries `check_api`, a few kB.

**Tests.**
- **Tier 3:**
  - `pnpm check` clean;
  - `pnpm fixtures` up to date;
  - `pnpm test`: Rust **1013 / 0 / 7** (the workspace, run), Vitest **744**;
  - `pnpm e2e --workers=4`: **57**;
  - `golden_frames` 21 (+1).
- **Media, on a keyed `pnpm build:e2e`:**
  - `image-parity.spec.ts` 1;
  - `pipeline-preview.spec.ts` 15 and `export-creator.spec.ts` 18: every session now decodes the images, and both pass, with the verifier's PASS on the Creator export.

**Declared.**
- `golden_frames.rs` gains lines only.
- `playwright.config.ts`' `MEDIA_SUITES` gains one line.
- `session.rs` and `protocol.ts` gain lines only, as the gate checks.
- No test of the base changed.

**Found.** From the Bash tool, `verifyMp4`'s plain `python` is MSYS2's 3.9 without NumPy, so `export-creator`'s verifier case failed on the first run ("NumPy cannot be imported"). With Python 3.13 first on `PATH` it passed. This is the machine's setup; nothing in the repository is changed.

**Push 2.** Pushed after this commit and waited on. CI's result (`golden_frames` on lavapipe, `image_basic` included) is recorded in the next prompt's entry.

**Open.** 35. Known issues 5 and 8 (P212, P210). Next: P199 with P198 (Bezier easing and the camera drift).

## Entry 60 - 2026-11-12 - P199 (with P198): bezier easing and the camera drift

**P197's push 2, read (CI run 43263410381, head `1f531d5`): green.** Jobs `changes`, `ci` and `e2e-media` succeeded (deploys skipped, as on a branch).
- `golden_frames` on lavapipe (llvmpipe, LLVM 21.1.8): `image_basic` **0.9997**, every golden at least 0.9996, 21 passed. TE-30 passes on both lines.
- gitleaks: 288 commits, no leaks (the bundle's SHA-256s pass).
- The media job: 24 passed and 29 passed (flags). Two `model-download` `@no-gpu` cases passed on their retry: known issue 3, seen again.

**What changed.**
- **Easing first (P198's part).** `easing.rs` gains, below `ease()`:
  - `CubicBezier` and `bezier(c, t)` (Newton's method, bisection where it fails, solved in f64);
  - `STANDARD` (0.4, 0, 0.2, 1), `OUT` (0, 0, 0.2, 1), `IN_OUT` (0.65, 0, 0.35, 1).
  - `Easing` and `ease()` are byte for byte as before (the gate's additive check).
- **`camera.rs`** (new, pure; D-476):
  - `CameraTrack` (`none()`, `scale_at`, `segs`), `CameraSeg`, `build(changes, sentences, duration)`, `for_clip(transcript, map)`;
  - `MAX_SEG_MS = 4,000`, `MIN_SEG_MS = 800`, `DRIFT = 0.05`.
- **`video.rs`** (D-477):
  - `Track` holds the camera and the source frame's size; `src_at(t)` is the crop at a scale of exactly 1.0, else the crop shrunk about its centre and clamped inside the frame;
  - `MAX_TOTAL_ZOOM = 1.35` caps it (the `PunchIn` keys join in P200).
- **`build_scene`** takes `Track::fixed(crop)` with `Enhance.motion` off (today's path) and the camera's track with it on.
- **Docs:**
  - TS §5 rows (`camera.rs`, `easing.rs`, `video.rs`, `tests/camera.rs`, `motion_mid`);
  - "As read (P199)" under D-475, D-476, D-477;
  - TE-31 and TE-30's lavapipe line in `experiments.md`.

**Differs from the plan (the As read notes).**
- `build` takes no `shots` (none exist before V6.5).
- The only change point is the clip's start: the hook's end joins when P205 draws the hook.
- The scale is continuous: each segment starts where the last ended, and a held segment (under 0.8 s) holds that level, not 1.0, so a split inside one shot never jumps.

**Who sees it.** Nobody yet. No web code sets `enhance` until P208's panel, so every build, flags on included, draws `v5()`: Motion off, today's frames.

**Tests (+21).**
- `easing.rs` +5:
  - ends exact, and NaN and outside held;
  - rising over 1,000 points;
  - **known values to 1e-4** at five points per curve, computed apart from the code (bisection to 1e-13 in Python); moving one control point by 0.02 fails it;
  - Newton and bisection agree to 1e-5 (Newton answers more than 900 of 999 points);
  - `IN_OUT` symmetric.
- `video.rs` +5:
  - no drift equals today at every frame of a 60 s clip, `src` bit for bit;
  - a scale of exactly 1.0 gives the crop bit for bit;
  - the zoom is about the centre;
  - the clamp at the frame's edge;
  - the cap.
- `tests/camera.rs` (new, 10):
  - `none()`; no split up to 4 s; the split at the nearest sentence start, repeated, the earlier of two;
  - a long stretch with no start stays whole; changes as bounds;
  - alternation without a jump; the held short segment; the curve and the range; determinism;
  - **the six labeled cases with motion off and on: at every frame, only `Video.src` differs** (inside the crop, smaller by at most the drift), and off is today's.
- `golden_frames.rs` +1, **`motion_mid`**: Clean's NumberReveal with motion on at the middle of the first segment (scale about 1.03, asserted). Written once on WARP, read beside the same instant with motion off: the video zoomed about its centre, the captions and the visual unmoved.

**TE-31: passes** (`experiments.md`).
- 150 frames (5 s at 30 fps) of the drift, drawn on WARP and read to a fraction of a pixel.
- The frame follows the scale to within 0.019 px.
- In 136 of the frames the expected move is over 0.005 px. There each frame moved 0.81 to 1.12 times its expected move (median 1.008), and none stood still then jumped.
- The fastest move is 0.14 output px a frame.

**Every existing frame unchanged.** No snapshot changed. Every old golden reads SSIM 1.0000 on WARP.

**Media (Tier 2):**
- keyed `pnpm build:e2e` and `pnpm build:e2e:flags`;
- `flags-identity.spec.ts` fails at its V6.1 pin of the flags-on database version (2; 3 since P192's `plans` store), **not from this prompt**: known issue 10, for P209 with D-440 (8);
- read with the pin temporarily at 3, the whole case passes (the same records, and the same export from both builds), and the spec file was restored unchanged.

**Render binary.** 7,496,980 → **7,507,007** bytes (+10,027: the curves and the camera).

**Counts.**
- Rust **1034 / 0 / 7** by count (1013 + 21; `offcut-scene` 122, `offcut-render` 34, `offcut-wasm-render` 12 re-run).
- Vitest **744** (no web code).
- `pnpm check` is clean; the gate prints `gate: ok`.

**Open.** 35. Known issues 5, 8, 10 (P212, P210, P209). Not pushed (push 3 is P206). Next: P200 (`PunchIn`).

## Entry 61 - 2026-11-13 - P200: PunchIn

**What changed.**
- **`events/punch_in.rs`** (new; D-478): `Track { span, scale, motion }`, `layout`, `scale_at`, and a `draw` that adds nothing (as `KeywordPop`'s):
  - `Snap` is at the scale from its first word's program start over its anchors;
  - `Push` rises from 1.0 on `IN_OUT` across them;
  - both release to 1.0 over `ZOOM_RELEASE_MS = 300` on `OUT`;
  - `MILD = 1.12`, `STRONG = 1.25`.
- **`events/mod.rs`:** `EventTrack::PunchIn`, its `kind`, its layout arm (the item's program span) and its `draw` arm. `PunchIn` leaves the "not drawn yet" arm.
- **`video.rs`:** `Track::with_punches` and `Track::scale_at`. The scale is the drift times the strongest zoom at `t` (two zooms never multiply), capped at `MAX_TOTAL_ZOOM = 1.35`, never below 1.0.
- **`lib.rs`:** `video_track` hands the plan's zooms to the video track.
  - Motion off and no zoom: `Track::fixed`, today's path.
  - **The plan's zooms stay with motion off** (§12: "the plan's zooms stay"); the switch is the camera's only.
- **Tests' `common/model.rs`** (new): `with_model_items`, a model's plan of given items validated for a case. It is its own file, so the binaries that do not use it do not warn.
- **Docs:** TS §5 rows (`events/punch_in.rs`, `tests/punch_in.rs`, `common/model.rs`, the goldens); "As read (P200)" under D-478.

**Tests (+14).**
- `punch_in.rs` +5:
  - Mild and Strong;
  - a snap in at its first word, held, released over 300 ms on `OUT`, falling all the way;
  - a push on `IN_OUT`, half way at half time;
  - an empty span only releases;
  - it draws nothing.
- `video.rs` +3:
  - a zoom multiplies the drift, and alone it leaves the crop bit for bit before and after;
  - two zooms whose release and start touch take the stronger, never the product;
  - the product capped at 1.35.
- `tests/punch_in.rs` (new, 4; a model's plan of one Strong zoom on "3,000 users"):
  - a snap held over its anchors and released in 300 ms with motion off, about the centre, at every frame;
  - a push rising across its anchors;
  - **the zoom changes only `Video.src`**, switched off (`event_overrides`) it is gone, and it is one visual moment of the summary;
  - with motion on, the drift times the zoom at every frame, never past the cap.
- `golden_frames.rs` +2:
  - **`punch_snap`** (×1.25 at the middle of its anchors) and **`punch_push`** (×1.125, half way), motion off, the scale asserted before the golden;
  - written once on WARP, read side by side: the video zoomed about its centre, the captions unmoved.

**Declared.** `golden_frames.rs` gains lines only (a module line and the two tests). No test of the base changed. P199's `video.rs` tests are unchanged (`with_punches` is additive).

**Every existing frame unchanged.**
- No snapshot changed. Every old golden reads SSIM 1.0000 on WARP.
- **`punch_in_110.png` is byte for byte its committed file.** The update run's name filter (`punch_`) also matched it: it was rewritten identically, then checked out from git and its hash compared.
- Media: on a keyed `pnpm build:e2e`, `pipeline-preview`'s three preview cases pass.
- No browser flow can draw a `PunchIn` yet: model plans arrive through `/plan`, which the fake API lacks until P210.

**Render binary.** 7,507,007 → **7,507,821** bytes (+814).

**Counts.**
- Rust **1048 / 0 / 7** by count (1034 + 14; `offcut-scene` 134, `offcut-render` 36, `offcut-wasm-render` 12 re-run).
- Vitest **744**.
- `pnpm check` is clean; the gate prints `gate: ok`.

**Open.** 35. Known issues 5, 8, 10 (P212, P210, P209). Not pushed (push 3 is P206). Next: P201 (Glow and the Glow style).

## Entry 62 - 2026-11-13 - P201: glow and the Glow style

**What changed.**
- **The glow (D-479):**
  - `styles.rs`: `Glow { layers, spread, alpha }` with `reach`, and `StyleSpec.glow`, `None` for Clean, Bold and Mono. Its reach is counted in `decoration_width`/`decoration_height` as a `max` with half the stroke, so a style with none lays out bit for bit as before.
  - `captions_draw.rs`' `draw_glow`, after the shadow and before the words: per layer, the widest first, each word's runs again as glyph runs with a clear fill and a stroke of the word's own colour at `alpha`.
  - `display_list.rs`: `Rgba::with_alpha`, so no colour literal leaves the files `styles_use_theme.rs` allows.
- **The Glow style (D-480):** `StyleId::Glow` appended (`edit.rs`, additive; the default stays Clean).
  - Its spec: Inter 800 at 76 lp as spoken, 2 lines of 14 characters, 3 words, no outline, Clean's drop shadow, the glow (4 strokes of 24, 18, 12 and 6 lp at alpha 41, reaching 12 lp), the spoken word highlighted in the accent, cut in hard, a keyword in `AccentAlt`.
  - Its theme is Clean's. **Each word glows in its own colour:** white words white, the spoken word and the keyword yellow.
- **Web:**
  - the picker's fourth card (`messages.plan.glow`, and `glowLine` in `copy/plan.ts`, a draft for CP 3: "Bright and soft: white words that glow, the keyword in colour.");
  - **the cards in two rows of two** (`panel.module.css`), so each keeps its width;
  - the worker draws four thumbnails (`THUMBNAIL_STYLES`);
  - `pnpm gen:types` (`StyleId` gains `"glow"`);
  - comments in `StylePanel.tsx` and `style-thumbnails.ts`.
- **Docs:** TS §5 rows (`styles.rs`, `captions_draw.rs`, `StylePanel.tsx`); "As read (P201)" under D-479, D-480 and D-440 (20).

**Differs from the plan.**
- `Glow` has no `role` (D-479 named one): the word's own colour is D-480's "keyword in the accent colour with glow" in one rule, with no new theme role.
- `protocol.ts`' doc line still says "three frames" (an additive file; its wording is left).
- The analytics' `style` is the `StyleId` type, so `"glow"` is accepted with no list changed.

**Tests (+8).**
- `edit.rs` +1: Glow serializes as `"glow"`, and the default is Clean.
- `styles.rs` +1: only Glow has a glow; its values; its decoration room (12 lp + the shadow).
- `theme.rs` +1: Glow's theme is Clean's.
- `captions_draw.rs` +2:
  - four strokes a word's run, the widest first (24 to 6), each the word's colour at alpha 41, white and yellow, all before the words;
  - Clean, Bold and Mono draw no glow.
- `layout_safe_area.rs` +1: **Glow over the 31 labeled files at three placements, watermark on: 79,059 frames and 748,344 glow strokes, each box counting its stroke's width; none leaves the safe area or meets the watermark.**
- `display_list_snapshots.rs` +1: Glow with each kind before, during and after: **12 new snapshot files**. `glow_keyword_pop_during` read command by command: the shadow layer, four glow layers, the words, the keyword at 1.25.
- `golden_frames.rs` +1, **`glow_style`**: the KeywordPop input during its pop. Written once on WARP and read beside `clean_keyword` and at 200%: a soft, graded halo with no hard ring; "brutal" yellow on a yellow glow.

**Declared.**
- D-440 (20): `review-edit.spec.ts` case 14 counts four cards, not three (`toHaveCount` and the observer that times them, and its words). The 400 ms budget and every other assertion are unchanged.
- Other test files gain lines only.

**Every existing frame unchanged.** No snapshot of Clean, Bold or Mono changed. Every old golden reads 1.0000. `styles_use_theme.rs` passes with Glow.

**Media** (keyed `pnpm build:e2e`, D1):
- **the four cards came 72 ms after the tab's click** (budget 400);
- `review-edit.spec.ts` 16 and `export-free.spec.ts` 12 passed (30 with the setup), axe clean on the editor.

**Captures** (`pnpm screens`, development build): `~/v6.2-review/ui/style-tab-glow-1280x800.png` and `-1920x1080.png`.
- At 1920 × 1080 the four cards and Highlight numbers fit.
- **At 1280 × 800 the second row's lines and the Highlight numbers switch are below the fold** (the panel scrolls). The layout is the founder's at CP 3.

**Render binary.** 7,507,821 → **7,508,635** bytes (+814).

**Counts.**
- Rust **1056 / 0 / 7** by count (1048 + 8; `offcut-types` 94, `offcut-scene` 140, `offcut-render` 37, `offcut-wasm-render` 12, `offcut-api-types` 33 re-run).
- Vitest **744**.
- `pnpm check` is clean (generated files staged first); the gate prints `gate: ok`.

**Open.** 35. Known issues 5, 8, 10 (P212, P210, P209). For CP 3: Glow's card line, and the style cards at 1280 × 800. Not pushed (push 3 is P206). Next: P202 (`Emphasis`).

## Entry 63 - 2026-11-13 - P202: emphasis

**What changed.**
- **`events/emphasis.rs`** (new; D-481): `Track { word, treatment, size, start }`, `layout` (the word from its program start), and a `draw` that adds nothing, as `KeywordPop`'s. `events/mod.rs`: `EventTrack::Emphasis`, its `kind`, layout and `draw` arms; `Emphasis` leaves the "not drawn yet" arm.
- **`lib.rs`:** `build_scene` hands each Emphasis to the captions as a `captions_draw::Mark` in the scene's new `marks` list, beside `Emphasis` (two V4 tests build that struct field by field, so it gains no field). **A word the captions do not show is not marked, has no track and is not counted** (a model's plan is kept on word edits, P193). `frame_at` and `caption_words` read the marks.
- **`captions_draw.rs`** (559 lines above its tests): `Mark`, `mark_at`, `marked_role`, `underline`, `highlight`, `draw_marks`, and `draw_marked`; `draw_with` is `draw_marked` with no mark.
  - **`Colour`:** the word in `Accent`, after the spoken word and a KeywordPop, before a number (D-169's first layer wins).
  - **`Underline`:** a bar in `Accent`, 0.10 em below the baseline, 0.08 em thick, round ends, inside the glyph box.
  - **`Highlight`:** the pill's box at rest (12 x 4 lp, radius 14) in `Accent`, padding shrunk at the safe area's edge; the word in `OnAccent`, with no outline, shadow or glow, spoken or not.
  - Order: panel, shadow, glow, **underlines and highlights**, pill, words, cursor. Both grow with the word.
- **`summary.rs`:** an Emphasis counts as **captions emphasized** ("N words emphasized"), not as a visual moment.
- **Docs:** TS §5 rows (`events/emphasis.rs`, `tests/emphasis.rs`, `captions_draw.rs`, `summary.rs`, the goldens, with P201's `glow_style` that the row missed); TS §10.6's field comment; "As read (P202)" under D-481.

**Differs from the plan.**
- The marks are a list of their own, not a field of `Emphasis` (see above).
- **`size: Punch` is drawn with its treatment at the captions' size**; P203 gives it its chunk and its size.
- A mark ends with its chunk: in Mono and Glow "brutal" ends its chunk, so it is marked only while said.

**Tests (+20).**
- `events/emphasis.rs` +1: the track for each treatment and size; it draws nothing.
- `captions_draw.rs` +7: a mark not started or on another word draws as before (four styles, every 10 ms); the role order over 16 layer combinations and three active treatments; colour from its start to the chunk's last frame, a KeywordPop winning; the underline's place, size and order in four styles; the highlight's box, ink, no outline, shadow or glow, clear of both neighbours, in four styles; a spoken highlighted word in ink on Bold's pop and under Clean's sliding pill, both marks growing with it; a highlight at the safe area's edge.
- `summary.rs` +1: three emphases and a KeywordPop are 4 emphasized, a zoom 1 moment.
- `tests/emphasis.rs` (new, 6; a model's plan on "The first week felt brutal for everyone." and the tail): marked exactly from its start while its chunk shows, **never the second "week" in another chunk** (four styles, every frame); coloured in the accent; **a word an edit hid is neither drawn nor counted** (every frame equals the plan without it); switched off it is gone, on it counts 1; **only the captions change** (the video and every run's place the same, new rectangles on the word, in the safe area); a punch word draws as a normal one.
- `display_list_snapshots.rs` +1: **18 new snapshot files** (three treatments in four styles while "brutal" is said; and after it in Clean and Bold). `emphasis_highlight_bold_after` and `_underline_bold_after` read command by command.
- `layout_safe_area.rs` +1: **the 31 labeled files in four styles at three placements, watermark on, each with a model's plan of up to 12 emphases: 3,132 marks drawn, 316,236 frames, 0 problems.**
- `golden_frames.rs` +3, Bold at 2,600 ms, the fill and the marks asserted first: **`emphasis_colour`, `emphasis_underline`, `emphasis_highlight`**. Written once on WARP by exact name (only the three files appeared); read beside the frame without the emphasis and at 200%, with the other three styles from a scratch render.

**Declared.** No test of the base changed. `display_list_snapshots.rs`, `layout_safe_area.rs` and `golden_frames.rs` gain lines only (a module line and new tests).

**Every existing frame unchanged.** No existing snapshot or golden is modified; every old golden reads SSIM 1.0000 on WARP; parity 0.9996-0.9998.

**Media** (keyed `pnpm build:e2e`, D1): `pipeline-preview.spec.ts`' preview cases, 5 passed. No browser flow can draw an Emphasis yet: model plans come through `/plan`, which the fake API lacks until P210.

**Render binary.** 7,508,635 → **7,509,799** bytes (+1,164).

**Counts.**
- Rust **1076 / 0 / 7** by count (1056 + 20; `offcut-scene` 157 / 0 / 4, `offcut-render` 40, `offcut-wasm-render` 12, `offcut-types` 94, `offcut-api-types` 33 re-run).
- Vitest **744**. `pnpm e2e` (non-media) not re-run: no web file changed.
- `pnpm check` is clean; the gate prints `gate: ok`.

**Open.** 35. Known issues 5, 8, 10 (P212, P210, P209). **For CP 3:** in Clean a highlight looks like the spoken word's pill (both yellow); in Bold, Mono and Glow a coloured word is the spoken word's colour; in Mono the cursor joins a spoken word's highlight; in Bold a highlight sits close to a neighbour as it pops. Not pushed (push 3 is P206). Next: P203 (punch words).

## Entry 64 - 2026-11-13 - P203: punch words

**What changed.**
- **`captions.rs`** (D-482; 455 lines above its tests): `PUNCH_SCALE = 1.9`, `chunk_punched(words, sentences, look, block, punches)` and `punch_line`. `chunk` keeps its signature and calls `chunk_punched` with no punch word, so the frozen `line_breaks.rs` and every V4 caller are untouched.
  - A punch word is a hard break before and after it: a chunk alone; the rest of its sentence is laid out by the same search on each side. No other sentence's chunks change.
  - In capitals in every style. **Its size is its line's `scale`** (the runs shaped at the style's size), which every part of the drawing already follows: glyphs, outline, Clean's pill, Mono's panel and cursor, Glow's glow, P202's marks, word boxes and hits. `captions_draw.rs` is unchanged.
  - Too wide at 1.9 for the block less the decoration grown 1.9 times: as large as fits. Down the block as `lines` places one line (DC-13's lowering), the decoration inside.
- **`lib.rs`:** `build_scene` takes the punch words before chunking: each switched-on `Emphasis { size: Punch }` whose word the captions show. Switched off or hidden: the chunks are as without it. One `shown` closure now serves both loops.
- **`events/emphasis.rs`:** its doc says where `Punch` goes now.
- **Docs:** TS §5 rows (`captions.rs`, `tests/punch_word.rs`, the goldens); "As read (P203)" under D-482 and D-440 (21).

**Differs from the plan.** None of substance. `captions_punch.rs` was not needed (455 lines, under 650). The reading floor needed no code: `hold_for_reading` runs on every chunk.

**Tests (+12).**
- `captions.rs` +4:
  - with no punch word, `chunk_punched` is `chunk` (four styles, three placements), and a punch word the captions do not hold changes nothing;
  - a punch word alone, every word once in order, in capitals at 1.9, shaped at the style's size, centred, its box and decoration in the block (four styles, three placements);
  - "internationalization" drawn as large as the block holds;
  - at a sentence's start or end, alone too.
- `tests/punch_word.rs` (new, 5; a model's plan over written sentences, four styles):
  - **the reading floor:** with silence after it, shown exactly 700 ms; with the next word at once, shown until the next chunk (470 ms), every word whole;
  - **two punch words 6 s apart** each alone, and a third 3 s after the second dropped (D-452 counts from the end of the one before);
  - **at the clip's last word:** the last chunk, shown to the floor or the clip's end, in capitals;
  - only its own sentence chunked anew, and a normal emphasis changes no chunk;
  - switched off or hidden by an edit, every frame as without it; on, one emphasis of the summary.
- `layout_safe_area.rs` +1: **the 31 labeled files in four styles at three placements, watermark on, up to 4 punch words each: 1,416 kept, 15,972 frames with one, 0 problems.**
- `display_list_snapshots.rs` +1: **8 new snapshot files** (four styles, while said and the next chunk). Clean, Bold and Mono read command by command: Clean's pill around "BRUTAL" at 1.9 in ink; Bold's pop held to about 1.016 by the safe area (1.93); Mono's panel and a cursor 63.8 lp wide.
- `golden_frames.rs` +1, **`punch_word`** (Clean, while said; one caption run, "BRUTAL" at 1.9, asserted first). Written once on WARP by exact name (only the one file appeared); read beside the frame without it and at 200%, with the other three styles, its fade-in and the hand-over from a scratch render.

**Declared.** D-440 (21): `tests/emphasis.rs`' `a_punch_word_is_marked_at_the_captions_size_until_p203` (P202's, pinning P202's interim rule) becomes `a_punch_word_is_marked_with_its_treatment_in_a_chunk_of_its_own`. Its frame equality is kept for the other sentence; it gains the punch word alone, in capitals, 1.9 times as tall, still marked. The equality over the punch word's own sentence cannot stay: D-482 changes those frames by design. Other test files gain lines only.

**Every existing frame unchanged.** `line_breaks.rs` passes unchanged; no existing snapshot or golden is modified (P202's 18 emphasis snapshots included); every old golden reads SSIM 1.0000 on WARP; parity 0.9996-0.9998.

**Media** (keyed `pnpm build:e2e`, D1): `pipeline-preview.spec.ts`' preview cases, 5 passed. No browser flow can draw a punch word yet (model plans come through `/plan`; the fake API has it from P210).

**Render binary.** 7,509,799 → **7,512,492** bytes (+2,693).

**Counts.**
- Rust **1088 / 0 / 7** by count (1076 + 12; `offcut-scene` 168 / 0 / 4, `offcut-render` 41, `offcut-wasm-render` 12, `offcut-types` 94, `offcut-api-types` 33 re-run).
- Vitest **744** (no web file changed; not re-run). `pnpm e2e` (non-media) not re-run.
- `pnpm check` is clean; the gate prints `gate: ok`.

**Open.** 35. Known issues 5, 8, 10 (P212, P210, P209). **For CP 3** (besides Entry 63's): a punch word followed at once by speech is on screen only for its own word (about 0.4 to 0.5 s); Glow's halo and Mono's cursor grow with it. Not pushed (push 3 is P206). Next: P204 (emoji and icons).

## Entry 65 - 2026-11-13 - P204: emoji and icons

**What changed.**
- **`events/emoji.rs`** (new; D-483): the track (word, image key, place, start) draws nothing in `events::draw`: the emoji belongs to the captions (the catalog's zone). `place` sets it once by its chunk's layout boxes into a `Placed`; `draw_placed` pops it in and stops at the chunk's end.
  - Size: 1.6 times the capitals' height of its line, never above its 160 px.
  - **`Under`:** below the chunk's lines, centred on the word. **Over the lines when that would reach the watermark's band or leave the safe area: at Bottom, the default, always.**
  - **`Beside`:** at the end of the word's line, centred on the capitals, if it fits in the safe area and is clear of every word of the chunk; else as `Under`.
  - Each place keeps the decoration, a 16 lp gap and a KeywordPop's growth off the words.
- **`events/icon_pop.rs`** (new): a 168 lp disc in `Accent` with the 112 lp white icon, drawn through `events::draw`; `layout` centres it in the zone, `attach` sets it on its word (across, kept in the zone; down, on the zone's edge by the captions) and the chunk's end.
- **`events/mod.rs`:** `EventTrack::{Emoji, IconPop}` and their arms; `image_pop` (0.6 → 1.0 over 120 ms on `OUT`) and `about_centre`. `HookTitle` is the last kind not drawn.
- **`layout.rs`:** `cap_height`, parley's `cap_height` of "H" (0.7 of the size if a face says nothing).
- **`lib.rs`:** `build_scene` places each emoji (into the scene's `emojis`) and attaches each icon by the chunk that shows its word; a word not shown, or an emoji with no place, has no track and no count. `frame_at` draws the emoji after the captions. **`testing::bounds` measures an `Image`** (D-440 (4)).
- **Docs:** TS §5 rows (`events/emoji.rs`, `events/icon_pop.rs`, `layout.rs`, `tests/emoji_icon.rs`, the goldens); "As read (P204)" under D-483, D-471 and D-440 (4).

**Differs from the plan.**
- **`ImageSet` is not added to `SceneInput`** (D-471 planned it): the scene emits only keys, every key exists (`image_bundle.rs`), the renderer holds the set since P197, and a new field would change every test that builds `SceneInput`.
- `Under` is below the chunk's lines, not the 280 lp block, and over them without room (D-483 had no fallback for `Under`).
- The icon also leaves with its word's chunk (D-483 named the exit for both).

**Found and fixed in the prompt.** The safe-area sweep's first run found 1,114 frames of an emoji over a caption word (three cases at Top): `Beside` beside a short first line sat over a wider second line. `Beside` now needs to be clear of every word of the chunk; the run after: 0.

**Tests (+16).**
- Unit +6:
  - `emoji.rs` 2: the track draws nothing; the placed emoji pops on `OUT` about its centre and stops at its end;
  - `icon_pop.rs` 2: disc and icon, their pop and times; `attach` centred on the word, kept in the zone, on the edge by the captions;
  - `layout.rs` 1: Inter's and JetBrains Mono's capitals;
  - `lib.rs` 1: `bounds` of an image.
- `tests/emoji_icon.rs` (new, 6; four styles):
  - under below the lines at Top and Middle, over them at Bottom, never on a word, in the safe area and off the band;
  - beside when the line leaves room, else under;
  - the pop, and shown exactly while the word's chunk is;
  - the icon's circle in the accent, in the event zone, centred on the word, on the edge by the captions, leaving with the chunk;
  - **only images and the disc are added to a frame**; hidden or switched off, every frame as without; two visual moments;
  - on a punch word, 1.9 times as large, capped at 160.
- `layout_safe_area.rs` +1: **the 31 labeled files in four styles at three placements, watermark on, an emoji under, an emoji beside and an icon in turn every 2.7 s: 3,056 kept, 75,237 images drawn, none outside the safe area, on the watermark or on a caption word.**
- `display_list_snapshots.rs` +1: **16 new snapshot files** (under at Middle, over at Bottom, beside, the icon; four styles). Three read command by command: Bold's fire under the second line centred on "BRUTAL", 97.8 lp; Bold's beside falling back under the full line; Clean's disc on the zone's bottom edge with key 1009.
- `golden_frames.rs` +2, the list asserted first: **`emoji_under`** (Bold at Middle) and **`icon_pop`** (Clean at Bottom). Written once on WARP by exact name (only the two files appeared); read by eye, with the four styles at three placements from a scratch render at 200%.

**Declared.** D-440 (4): `testing::bounds`. Other test files gain lines only.

**Every existing frame unchanged.** No existing snapshot or golden is modified; every old golden reads SSIM 1.0000 on WARP; parity 0.9996-0.9998.

**Media** (keyed `pnpm build:e2e`, D1): `pipeline-preview.spec.ts`' preview cases and `image-parity.spec.ts`, 6 passed; TE-30 again 0.9995, the 72 images decoded in 19 ms. No browser flow can draw a plan's emoji or icon yet (model plans come through `/plan`; the fake API has it from P210).

**Render binary.** 7,512,492 → **7,522,176** bytes (+9,684).

**Counts.**
- Rust **1104 / 0 / 7** by count (1088 + 16; `offcut-scene` 182 / 0 / 4, `offcut-render` 43, `offcut-wasm-render` 12, `offcut-types` 94, `offcut-api-types` 33 re-run).
- Vitest **744** (no web file changed; not re-run). `pnpm e2e` (non-media) not re-run.
- `pnpm check` is clean; the gate prints `gate: ok`.

**Open.** 35. Known issues 5, 8, 10 (P212, P210, P209). **For CP 3** (besides Entries 63 and 64): at Bottom, the default, an emoji "under" its word is drawn over it; **the white icons on Clean's, Bold's and Glow's yellow disc are weak in contrast** (Mono's green is fine; a dark disc or an accent ring are the alternatives); at Bottom the icon sits about 120 lp above the captions. Not pushed (push 3 is P206). Next: P205 (the hook title).

## Entry 66 - 2026-11-13 - P205: the hook title

**What changed.**
- **`lockup.rs`** (new, pure; D-484, §10.4): `lay_out(text, accent, (width, height), fonts)`. The hook's words in Inter 800 capitals (`LOCKUP_FONT`).
  - Every split into 2, then 3, lines is tried. **Each line takes its own size**: the largest from 140 lp down to 48 lp in steps of 4 at which it fits the width, the accent word at twice its line's.
  - Least worst distance from the width wins; ties go to fewer lines, then the first split.
  - Lines are stacked by their capitals, 0.25 of the smaller size apart; the lockup must fit the room.
  - Words are measured once at 100 lp and scaled. A one-word hook, a bad accent, or a line too wide at 48 lp gives `None`.
- **`events/hook_title.rs`** (new): the lockup centred in the safe area above the watermark's band; the theme's `Text`, the accent word in `Accent`, over a drop shadow of Clean's grown with the size; entering 0.9 → 1.0 over 150 ms on `OUT`; shown over `[0, 2 s)` of program time or a shorter clip's length.
- **`events/mod.rs`:** `EventTrack::HookTitle`; the item carries its text (`EventParams::HookTitle { text, accent }`), so it is laid out like any kind. **Every kind of the catalog is now drawn.**
- **`lib.rs`:** `pub mod lockup`; `Scene.hook_until`; `shown_chunk`: **under the hook no caption is drawn, hit or boxed, nor its emoji**.
- **`display_list.rs`:** `DrawCmd::scale` public, so that a test can compose a list as `frame_at` does.
- **Docs:** TS §5 rows (`lockup.rs`, `events/hook_title.rs`, `display_list.rs`, `tests/{lockup,hook_title}.rs`, `common/hook.rs`, the goldens); "As read (P205)" under D-484 and §10.4.

**Differs from the plan.**
- `lay_out` takes the room's width and height and returns `Result<Option<..>>`: the height rules out a split too tall, and a font error is not a hook that does not fit.
- D-484's "within 10% of the safe width" is what the search reaches, not a rule: two short words cannot ("STOP / SCROLLING" leaves 58%). Hooks of 5 words or more reach 0.7-3.1%.
- **The two goldens' lists are composed, not built by a scene:** a hook plan needs a `Label`, which `offcut-render`'s tests cannot build without a new dependency (§3.1). `hook_title.rs` proves a scene's frame under the hook equal to that composition at 1080 and at 540.

**Tests (+18).**
- `events/hook_title.rs` +3:
  - its colours and centre, until 2 s or a shorter clip;
  - the entry on `OUT`, shadow then runs, gone at its end;
  - a hook that cannot fit has no track.
- `tests/lockup.rs` (new, 7):
  - widths linear in the size (to 1e-3);
  - the face is Inter 800;
  - each line as large as fits (one size up does not), the accent at 2×, capitals, every word once;
  - **against a brute force that shapes every line at every size: the same worst distance and the same number of lines** for 8 hooks;
  - long hooks within 10%;
  - one word, a bad accent, a word too wide, an accent too wide, no room: `None`;
  - the same lockup twice.
- `tests/hook_title.rs` (new, 4; four styles):
  - **for 2 s each frame is the video and the hook exactly, then the frame without the hook exactly** (1080 and 540);
  - no caption word boxed or hit under it;
  - a hook that cannot fit, or switched off: every frame as without, not counted; on, one visual moment;
  - a clip under 2 s shows it throughout.
- `layout_safe_area.rs` +1: **12 hooks with each word the accent, four styles, three placements, watermark on: 804 of 816 drawn (the 12 others, one accent word too wide at 96 lp, not drawn), 676,287 boxes, 0 problems.**
- `display_list_snapshots.rs` +1: **8 new snapshot files** (four styles, entering and at rest). Clean read command by command: "THE ONE THING" at 112, "NOBODY" at 120 in the accent beside "TELLS YOU" at 60, the shadow layer first, 0.9 when entering.
- `golden_frames.rs` +2: **`hook_short`** ("STOP / SCROLLING") and **`hook_long`** ("THE ONE THING / NOBODY TELLS YOU / ABOUT SAVING"), Clean at 1,000 ms. Written once on WARP by exact name (only the two files appeared) and read by eye.

**Declared.** No test of the base changed. `display_list_snapshots.rs`, `layout_safe_area.rs` and `golden_frames.rs` gain lines only.

**Every existing frame unchanged.** No existing snapshot or golden is modified; every old golden reads SSIM 1.0000 on WARP; parity 0.9996-0.9998.

**Media** (keyed `pnpm build:e2e`, D1): `pipeline-preview.spec.ts`' preview cases and `image-parity.spec.ts`, 6 passed (TE-30 0.9995). No browser flow can draw a hook yet (model plans come through `/plan`; the fake API has it from P210).

**Render binary.** 7,522,176 → **7,534,119** bytes (+11,943).

**Counts.**
- Rust **1122 / 0 / 7** by count (1104 + 18; `offcut-scene` 198 / 0 / 4, `offcut-render` 45, `offcut-wasm-render` 12, `offcut-types` 94, `offcut-api-types` 33 re-run).
- Vitest **744** (no web file changed; not re-run). `pnpm e2e` (non-media) not re-run.
- `pnpm check` is clean; the gate prints `gate: ok`.

**Open.** 35. Known issues 5, 8, 10 (P212, P210, P209). **For CP 3** (besides Entries 63 to 65): the lockup is centred in the frame, over a speaker's face for its 2 s. **For P206:** `catalog_complete.rs`' harness item gains the five kinds; a caption-zone kind (Emphasis, Emoji) draws nothing in `events::draw`, IconPop draws in the event zone, PunchIn nothing (the video's), HookTitle in the safe area. Not pushed (push 3 is P206). Next: P206 (everything together, CP 3, push 3).

## Entry 67 - 2026-11-13 - P206: everything together, checkpoint 3; push 3

**What changed.**
- **`catalog_complete.rs` un-ignored (D-440 (9))**: the release's only ignores are gone (`git grep 'ignore = "V6.2' crates` is empty). The harness is widened, nothing deleted:
  - `KINDS` 4 → 9, and `item` makes one of each new kind;
  - the zone names gain `video` and `hook`, and `points` measures an `Image`;
  - each kind is drawn at 1,500 ms as well as 3,300 ms (the hook shows in the program's first 2 s);
  - a caption- or video-zone kind draws nothing of its own; every other kind draws inside its zone.
- **`tests/common/all_kinds.rs`** (new): the reference transcript (`a01-scriptA`) with a model's plan of a hook, four emphases (one a punch word), two emoji, two icons, a snap and a push, every item kept, Motion on.
- **New cases** (with all five kinds, four styles):
  - `determinism.rs`: two builds equal, the same bytes at every frame and in any order;
  - `parity.rs`: 540 and 720 the 1080 list after scaling, at every frame;
  - `display_list_snapshots.rs`: Glow at ten instants (10 new files, each read to show its kind).
- **The frame cost** (§10.5): `scene_timing_all_kinds` (ignored, release only). **0.78-1.23× today's per frame** (Mono's the worst, under the 1.5× line), every frame in microseconds; recorded in `docs/v6.2/experiments.md`.
- **Captures** in `~/v6.2-review/look/`: each kind at rest in each style (40), the hook at three lengths, ten drift frames, and `README.md` with §12's copy and the points to judge.
- **The whole clip** (D-496), in `~/v6.2-review/cp3/`: the MP4 and `plan-annotated.md`. How it was run is "As read (P206)" under D-496: `dist-flags/` behind a scratch same-origin proxy to the local API, per-process values for that run only, the founder's real SMTP values in the local `.env`, sign-in by magic link.
- **Docs:** TS §5 rows (`common/all_kinds.rs`, `determinism.rs`); "As read (P206)" under D-440 (9), D-496 and §10.5; open item 39.

**Checkpoint 3.** The founder's own recording (79 s), cloud transcription, **DeepSeek's plan (D-500)**, Glow, exported at the free plan's 720p.
- **The plan:** 15 items, **11 drawn** (3 zooms, 3 emphases one a punch word, 2 emoji, 2 icons, 1 number), 4 numbers dropped `NumberUnresolved`, **no `hook_title` item** (a hook text only).
- **Motion could not be turned on:** its switch is P208's Enhance panel, so the clip has the plan's zooms and no drift.
- **Found on the way** (not code faults; how the local run is set up):
  - the build verifies plan tokens with its test key, so the local API signed with the e2e test seed;
  - the cloud card needs `TRANSCRIBE_ENABLED`;
  - the landing page has no sign-in entry.
- **The founder's go, 2026-11-13, with no change asked.** The points raised stay open: open item 39 (twelve, from Entries 63-66 and this run).

**Differs from the plan.**
- The provider is DeepSeek, not Gemini (D-500).
- No drift in the clip (P208).
- `v62_kinds.rs` (§13.1) was built as one test file per kind (`emphasis.rs`, `punch_word.rs`, `emoji_icon.rs`, `hook_title.rs`, `lockup.rs`).

**Tests (+6, 3 un-ignored).** `catalog_complete.rs` 3 now run; `determinism.rs` +1 (+1 ignored timing); `parity.rs` +1; `display_list_snapshots.rs` +1.

**Declared.** D-440 (9): `catalog_complete.rs` as above. Other test files gain lines only (`display_list_snapshots.rs`' `labeled` module carries `#[allow(dead_code)]`: only its `case` is used there).

**Every existing frame unchanged.** No existing snapshot or golden is modified; every golden reads SSIM 1.0000 on WARP (32 readings), parity 0.9996-0.9998.

**Tier 3** (D1):
- `pnpm check && pnpm fixtures && pnpm test` exit 0 (`.env` loaded for the server's database tests);
- `pnpm e2e --workers=4`: 57 passed;
- `golden_frames`: 33 passed.

**Render binary.** 7,534,119 bytes (no scene code changed).

**Counts.**
- Rust **1128 / 0 / 5** (`cargo test --workspace`: 1122 + 6; the ignores 7 - 3 + 1; `offcut-scene` 204 / 0 / 2).
- Vitest **744**. `pnpm e2e` 57.

**Open.** 35, 39. Known issues 5, 8, 10 (P212, P210, P209). **Push 3** (not waited on). Next: P207 (the levelled voice in the DSP).

## Entry 68 - 2026-11-13 - The plan gains P206a to P206c (the founder's CP 3 changes)

**What changed.** Docs only. After the go, the founder asked for three of checkpoint 3's points to be fixed (open item 39 (8), (11), (12)). As CP 3's rule says, they become prompts before P207:
- **P206a**, a way to sign in from the landing page (D-501). The fallback line's "Sign in" opens the dialog in place: leaving the editor would lose the clip before V7.
- **P206b**, icons on a dark disc ringed in the accent (D-502). The icon goldens and snapshots are regenerated by name; the founder sees the new icon before the commit.
- **P206c**, the hook title always shown when the AI writes a hook (D-503): `validate` adds the item a model left out (no AI call), and the prompt asks for it. The prompt's re-read (about $0.02) waits for the founder's yes.

Written into `coding-promptsv6.2.md`, D-501 to D-503, §17's prompt table, and the build guide's Phase 6 table; open item 39 marks the three as planned.

**Open.** 35, 39. Not pushed (push 4 is P209). Next: P206a.

## Entry 69 - 2026-11-13 - P206a: sign in from the landing page; push 3's CI read

**What changed.**
- **The landing page's top bar** gains one link to `/account`: "Sign in", or "Account" once this tab is signed in (`ui/landing/account-link.ts`, its words in `messages.landing.account`). It sits after the section links and stays on a phone, where they are hidden. `LandingPage` and `TopBar` take `accountPath` (the route passes `ROUTES.account`). Its styles join the section links' rules in `landing.module.css` (687 lines).
- **The AI plan's signed-out toast** ("Sign in to get an AI-planned edit…") gains a **"Sign in"** action (`ui/components/plan-fallback.ts`; the other reasons carry none). It sets `ui-store`'s new `signIn`, and `EditorLayout` draws `SignInDialog` over the editor, as it draws the shortcuts sheet. The page never leaves `/app`, so the clip stays. Once the tab is signed in, the dialog closes, as the export gate's does.
- **The dialog's title** from the toast is "Sign in for an AI-planned edit": `SignInDialog` gains an optional `title`, and "Sign in to export" stays the default for its two other callers.
- **Copy (drafts for the founder):** "Sign in", "Account" (the landing page); "Sign in", "Sign in for an AI-planned edit" (`copy/plan.ts`).
- **Captures** in `~/v6.2-review/look-fixes/` (`p206a-*`): the landing page at 1280 × 800 and 390 × 844, signed out and signed in; the toast with its "Sign in"; the dialog over the editor. They come from three new cases in `screens.spec.ts`. The editor's half processes the reference clip signed out with `ai_plan` on, clicks the toast's "Sign in", checks the dialog and its title at `/app`, then closes it with Escape and checks that the clip still plays.
- **Docs:** TS §5 rows for the two new modules and their tests; "As read (P206a)" under D-501; open item 39 (11) done; known issue 11.

**Differs from the plan.** The dialog's own title (the plan said only "opens `SignInDialog`"): opened from the plan's toast, "Sign in to export" read wrong.

**Push 3's CI** (run 43329295884 on `1c94d83`, read at this prompt's start):
- **Attempt 1 failed:** `golden_frames` died with SIGSEGV on lavapipe after 14 of 33 goldens had passed, each at 0.9997. Push 1's run 43185400050 had failed the same way and was not read at the time.
- **Attempt 2, one re-run of the failed job as the rule allows, passed in full:**
  - goldens on llvmpipe (LLVM 21.1.8): 33 passed, every one at least 0.9997 (`motion_mid`, `punch_push` 1.0000; `punch_snap`, `tech_list` 0.9999);
  - non-media 57 passed;
  - media 26 passed and flags 29 passed, no flaky case, one standing skip each.
- Recorded as known issue 11, to watch.

**Tests (+7).** Vitest +5 (`account-link.test.ts` 2, `plan-fallback.test.ts` 3); `landing.spec.ts` +2 (signed out: "Sign in" leads to the form; signed in: "Account", on a phone too). Every existing case unchanged.

**Tier 3** (D1, with P206b's uncommitted Rust in the tree; the two touch no common file):
- Vitest **749**;
- `pnpm e2e` **59 passed** (keyed `build:e2e`). In one earlier targeted run, L-11 (axe on the landing page) failed once under four workers; it passed alone, in the full file and in the full run;
- `pnpm check` exit 0.

**Open.** 35, 39. Known issues 5, 8, 10, 11 (P212, P210, P209, watch). Not pushed (push 4 is P209). Next: P206b (icons on a dark disc, waiting for the founder's yes on the look).

## Entry 70 - 2026-11-13 - P206b: icons on a dark disc

**What changed.**
- **`events/icon_pop.rs`** (D-502):
  - `Track` gains `panel` (`Role::Panel`) beside `color` (the accent).
  - `draw` pushes the accent disc (168 lp), the panel's disc 8 lp inside it (`ICON_RING`, 152 lp), then the white icon (112 lp). The pop scales all three about the disc's centre.
  - Every colour comes from a theme role, so Mono gets its green ring.
- **Regenerated by exact name** (V6.2's own files, not the base's):
  - `fixtures/golden/icon_pop.png`;
  - the snapshots `icon_pop_clean`, `icon_pop_bold`, `icon_pop_tech`, `icon_pop_glow`, `all_kinds_glow_icon_numbers` and `all_kinds_glow_icon_laptop`.
  - Each snapshot's diff is the inner disc's `FillRect` alone (17 lines); nothing else moved.
- **Review frames** in `~/v6.2-review/look-fixes/` (`p206b-*`, with a README): each style at Top and Bottom, today's icon and the new one side by side at 200%, over a still of the reference clip. "Today's" is the same list without the inner disc. A throwaway test drew them and was deleted.
- **Docs:** "As read (P206b)" under D-502 and D-440 (22); open item 39 (8) done.

**The founder's yes on the new look, 2026-11-13.** Panel is 85% opaque, so a little of the accent shows through the dark disc: dark olive in Clean, Bold and Glow, dark green in Mono. The founder liked it as it is.

**Declared (D-440 (22)).** Three checks widened, none deleted:
- `icon_pop.rs`' unit test (renamed `the_disc_is_ringed_in_the_accent_dark_inside_centred_in_the_zone_with_the_icon_on_it`) asserts the ring, the inner disc and their pop, keeping every other assertion;
- `emoji_icon.rs`' `without_images` also takes out a panel-coloured circle 100 lp across or more;
- `golden_frames.rs`' `icon_pop` reads the ring and the inner disc, with the icon inside the inner one.

**Every other frame unchanged.** No other golden or snapshot changed; `golden_frames` 33 passed on WARP. The safe-area sweep (`layout_safe_area.rs`' emoji and icon case) is green.

**Tests (0 new).** Three widened, as declared.

**Tier 3** (D1):
- Rust **1128 / 0 / 5** (`cargo test --workspace`, `.env` loaded; the same counts as Entry 67: nothing added);
- Vitest 749; `pnpm e2e` 59 passed (keyed `build:e2e`, with this change built in);
- `pnpm check` exit 0.

**Open.** 35, 39. Known issues 5, 8, 10, 11 (P212, P210, P209, watch). Not pushed (push 4 is P209). Next: P206c (the hook title always shown; the prompt's re-read, about $0.02, waits for the founder's yes).

## Entry 71 - 2026-11-13 - P206c: the hook title always shown

**What changed.**
- **The guarantee, in `validate` (D-503), with no AI call.**
  - A model plan with a valid `extras.hook` that kept no `HookTitle` after V-2 to V-5 gains one, checked by `check_item` like any item.
  - The added item: anchors `0..word count`; accent the hook's longest word by letters and digits (the first of equal ones); id one more than the plan's largest (1 for an empty plan). Nothing is reported as dropped for it.
  - It replaces a model's own hook item that fails too (TE-15 below found 5 in 15). A plan of the rules never gains one.
- **V-9 reads the hook title first** (`density_rule`). `limit` sorts by first word, then kind, and `HookTitle` sorts last, so a visual anchored on the clip's first word used to beat the hook. Only that tie changes; the hook opens the clip.
- **The instruction:**
  - the "finished short" line says "open with the hook title";
  - the extras line says to "always add a hook_title item with it";
  - the catalog row's purpose begins "Always, with the hook".
  - The system message is 16,261 characters (16,366).
- **The re-read** (the founder's yes, about $0.02): 15 Flash calls, $0.017, balance $1.81.
  - **The model wrote the hook item in 15 of 15 plans** (1 of 15 at P188); 10 of its own were valid.
  - The other 5 counted wrong (an accent past the hook's last word 3, anchors short of the clip's end 2), and the guarantee replaced each. **Every clip opens on its hook.**
  - Median drops 10.0%, at TE-15's line, from those 5 (0.0% without them, as P188c); p90 2.9 s; parse 15 / 15 first try.
  - In `experiments.md` (TE-15, "The hook line"); the annotated plans are in `~/v6.2-review/p206c/plans/`.
- **Docs:**
  - TS's two statements of the old rule now name D-503;
  - the implementation doc's "not checked" row is struck through and its catalog table row updated;
  - "As read (P206c)" under D-503 and D-440 (23);
  - open item 39 (12) done.

**Differs from the plan.**
- "Kept no `HookTitle`", not "has none": a model's broken hook item would otherwise leave the clip without its hook. The re-read showed this happens a third of the time.
- V-9's hook-first order is new: the plan assumed the added hook is always the first visual, and on a tie at the first word it was not.

**Declared.**
- **D-440 (23).** Three cases build a model plan with a valid hook to test the extras or the non-item fields: `validate.rs`' inline `every_non_item_field_is_dropped_in_v4`, `tests/validate.rs`' `the_non_item_fields_are_dropped_and_the_items_kept` and `offcut-wasm-render`'s `validate_plan_answers_the_text_extras_that_passed_and_only_those`. Each now counts the added hook beside its items (1 → 2, 5 → 6, 1 → 2), keeping every other assertion. `scene_api.rs` is additive, so `v62-gate.sh`'s ALLOW gains the one line it loses.
- **Changed by design:**
  - `catalog_snapshot__system_prompt@v2.snap` (three lines: the catalog row, the finished-short line, the extras line);
  - `server/tests/fixtures/deepseek/request.recorded.json`'s system length (16,366 → 16,261) and its test's comment.
- Every other validate case unchanged, the V6.2 hook and V-9 cases included.

**Tests (+6).**
- `validate_v62.rs` +5:
  - the hook added, with its accent and id;
  - V-9 after it, and a first-word visual giving way;
  - a plan's own valid hook untouched;
  - a failing hook item dropped and replaced;
  - nothing added without a valid hook, or to a plan of the rules.
- `hook_title.rs` +1: the founder's case (a hook, no item) draws every frame as the same hook written as an item would, the lockup over the first 2 s, in the four styles.

**Tier 3** (D1):
- Rust **1134 / 0 / 5 (1128 + 6)** (`cargo test --workspace`, `.env` loaded);
- Vitest 749; `pnpm e2e` 59 passed (keyed `build:e2e`);
- `pnpm check` exit 0.

**Open.**
- Items 35 and 39 stay open: 39's three fixes are done, its other points stay for the quality pass.
- Known issues 5, 8, 10 and 11 (P212, P210, P209, watch).
- Not pushed (push 4 is P209).
- Offered: a third prompt revision on the hook item's counting (about $0.02, the founder's call).
- Next: P207 (the levelled voice in the DSP).

## Entry 72 - 2026-11-13 - P206c, second commit: the hook item counted right

**What changed.** The third prompt revision Entry 71 offered, with the founder's yes. `prompt.rs`' extras line now spells out the hook item:
- its accent counts the hook's own words from 0, not the transcript's ("We cut our bill": "bill" is 3; the last of n words is n - 1);
- its anchors are every word of the transcript, start 0, end one past the last word's index.

The system message is 16,500 characters (16,261).

**The re-read** (15 Flash calls, $0.0175, balance $1.81 just after; `experiments.md`, TE-15, "The hook item's counting"):
- **The model's own hook item kept in 14 of 15 plans** (10 of 15 in Entry 71); the one miss, `w04`'s accent 4 on a 4-word hook, was replaced by `validate`'s item. Every clip opens on its hook.
- Parse 15 / 15 first try; p90 3.0 s.
- **The median drop rate is 11.1%, over TE-15's 10% line** (mean 10.7%, down from 13.1%). The drops over the line are all outside this revision:
  - numbers that do not resolve (9);
  - two visuals V-9 kept apart;
  - one zoom too close.
- Their paragraphs are unchanged since P188c, and the drops moved between transcripts from run to run. Read as the variance of 15 plans, not a regression; no further revision proposed.

**Declared.** Changed by design:
- `catalog_snapshot__system_prompt@v2.snap` (the extras line);
- `request.recorded.json`'s system length (16,261 → 16,500) and its test's comment.

**Tests.** No new or changed case. `offcut-plan` 80, `offcut-api` 334, `pnpm check` exit 0.

**Docs.** TE-15's new section; D-503's "As read (P206c)" gains (5).

**Open.** 35, 39. Known issues 5, 8, 10, 11 (P212, P210, P209, watch). Not pushed (push 4 is P209). Next: P207 (the levelled voice in the DSP).

## Entry 73 - 2026-11-13 - P207: the levelled voice

**What changed.**
- **`offcut-dsp/src/level.rs`** (new, D-487): `level_only(pcm48, &ChainConfig) -> Result<ChainOutput, DspError>`, the voice with Enhance's voice switched off.
  - It is loudness to the target, the limiter at the ceiling and one corrective pass: `run_chain`'s steps 4 to 7, written out again so that `run_chain` stays as it is.
  - No high-pass, denoise or compressor, and no progress callback.
  - It refuses what the chain refuses. The +24 dB gain cap is the chain's.
- **`assemble`'s voice.** `ProgramConfig` gains `voice: VoiceMode` (re-exported from `offcut_dsp::program`), `Clean` by default.
  - `assemble`'s signature is unchanged: its first argument is the voice the config names, `out48` for `Clean` and `lvl48` for `Level`.
  - `ProgramConfig`'s one-line empty struct is the one line `program.rs` loses; `v62-gate.sh`'s ALLOW names it.
- **WASM** (`offcut-wasm-core/src/audio_api.rs`, additive):
  - `level_only(pcm48)` answers `{ lvl48, loudness, peak }`;
  - `assemble_voice(voice48, voice, time_map, speech)` answers as `assemble` does; an unknown voice is `E_INTERNAL` `Voice`.
  - `run_chain`, `assemble` and `measure_prosody` are unchanged. Nothing in the page calls the new entries until P208.
  - **Read from Node on the built module:**
    - a -30 dBFS sine levelled to -14.00 LUFS;
    - the Level program bit for bit `lvl48`;
    - `"clean"` bit for bit the old `assemble`;
    - an empty input `E_DSP` `Empty`.
  - The core binary grew 3,917 bytes (804,508 → 808,425).
- **Docs:** TS §5 rows (`level.rs`, `tests/level_only.rs`, `program_identity.rs`' new line); "As read (P207)" under D-487.

**Differs from the plan.** D-487 names `chain::level_only`; the prompt and the build guide name `level.rs`, which keeps `chain.rs` without a diff. Followed the prompt.

**Acceptance.**
- `git diff -- crates/offcut-dsp/src/chain.rs` is empty.
- `program_identity.rs`' three old cases pass unchanged: no line of the file is removed, only an import and the new case added.

**Tests (+8).**
- `level.rs` 2 (what it refuses; silence stays at the floor).
- `tests/level_only.rs` 5, on two derived recordings and synthetic signals:
  - the length is the input's;
  - the loudness is the target as the meter reads it, and the gain cap holds;
  - no sample passes the ceiling;
  - where the limiter does not act, the output is the input times one gain, so nothing else is done;
  - `run_chain` on the same input is unchanged, bit for bit, and the input is untouched.
- `program_identity.rs` +1: with `voice: Level` the program on the identity map is `lvl48` bit for bit, at its loudness, and music is refused as before.

**Tier 3** (D1):
- Rust **1142 / 0 / 5** (`cargo test --workspace`, `.env` loaded; 1134 + 8);
- Vitest 749; `pnpm e2e` 59 passed (keyed `build:e2e`, the new WASM in it);
- `pnpm check` exit 0.

**Open.** 35, 39. Known issues 5, 8, 10, 11 (P212, P210, P209, watch). Not pushed (push 4 is P209). Next: P208 (the Enhance panel, `lvl48` in the audio worker, the export's voice).

## Entry 74 - 2026-11-13 - P208: the Enhance panel

**What changed** (all behind `motion`; with it off, every path is V6.1's).
- **`lvl48`** (D-487):
  - The audio worker's `runChain` makes the levelled voice from the same `pcm48` when the page asks with `level: true`. The page asks with `motion` on; the worker reads no flag.
  - `takeLevel()` hands it over, and `assembleVoice()` builds a program on a named voice.
  - `run-pipeline.ts` writes `clips/<id>/lvl48.f32` after `out48.f32`; `prog48.f32` stays the clean program, written as before.
- **`voice-program.ts`** (new): the program on a voice, in one place for the review, the pipeline and the export. Clean reads `prog48.f32`, as the export always did; Level reads `lvl48.f32` and assembles it.
- **`set-enhance.ts`** (new):
  - `setEnhance` is a review change in mode `scene`, one history step, with no `review_action`;
  - it is remembered under `meta.enhanceDefault` once the clip shows it;
  - `startEnhance` gives a new clip the remembered state, else the recommended `{ clean, motion: true }`;
  - `resetEnhance` is settings' "Back to recommended".
  - With the flag off, nothing is read, written or switched.
- **The voice swap** (`review-update.ts`): when the voice differs between the edit before and the new one (the switch, its undo, its redo), the program on the new voice replaces the paused preview's buffer (`control-preview.ts`' `swapAudio`) and the clip's `prog48`. A failure restores the edit before.
- **`EnhancePanel.tsx`** (new): "Enhance" under the Style tab's cards, Voice cleanup and Motion with §12's lines, disabled during an export. **Settings:** "Enhance", a line, "Back to recommended" and a line once pressed. The two lines are new drafts.
- **The export** reads `prog48.f32` for Clean, unchanged, and assembles `lvl48.f32` for Level.
- **`export_done`** (D-490): `items_kept`, `items_disabled` (with `ai_plan`, counted as the Visuals tab counts) and `enhance_off` (with `motion`).
  - They are optional on the server: a V6.1 page's event is read and stored as before. The server caps the counts and refuses a row named twice.
  - The event's description in "What leaves your device" gains "which Enhance switches you turned off".
- **Docs:**
  - TS §5 rows (four files), the OPFS row for `lvl48.f32`, the `export_done` row;
  - "As read (P208)" under D-440, D-486, D-487, D-488, D-490;
  - `experiments.md`, D-488's switch time.

**The switch time on D1** (D-488, Chrome, the flags-on keyed build, the reference clip; read from the page's own `performance` measures):
- paused, Voice 32-83 ms (to Level about 75, back to Clean about 35) and Motion 11-20 ms;
- while playing, Voice 86-139 ms, its pause and play included;
- **all inside the 150 ms budget**, the playing switch closest.

The same run: `lvl48.f32` had `out48.f32`'s size (14,343,360 bytes); undo took a switch back; an export with Voice cleanup off finished, its `export_done` carrying `enhance_off: ["voice"]`, `items_kept: 4`, `items_disabled: 0`. The scratch case was not committed. P209's `enhance-panel.spec.ts` is the lasting proof, reading the exported voice through a dev hook.

**Differs from the plan.**
- `export_done`'s fields are sent only with their flags. Otherwise a flags-off export would differ from V6.1's, and `export-creator.spec.ts` pins its props key for key.
- The switch sends no `review_action`: its kinds are the server's closed list, and D-490 counts the choice in `export_done`.
- `runChain`'s parameter gains `level?` rather than the worker reading the flag.
- "Back to recommended" has a heading and two lines; §12 drafted only its button.

**Declared.**
- `v62-gate.sh` ALLOW: `protocol.ts`' two `runChain` lines (D-487).
- `eslint.config.js` edges:
  - `set-enhance.ts` → `review-update.ts`;
  - `run-pipeline.ts` → `set-enhance.ts`;
  - `review-update.ts`, `run-pipeline.ts`, `start-export.ts` → `voice-program.ts`.
- D-440:
  - (5) the prop-enum table 20 → 21 rows;
  - (15) the TypeScript test 49 → 50 names;
  - the server test helper gains the three fields as `None`;
  - the fakes of `run-pipeline.test.ts` and `start-export.test.ts` gain the new methods, and `start-export.test.ts` mocks the flags at their defaults.
- No existing case changed.

**Known issue 10 (P209), wider.** `flags-identity.spec.ts`' flags-on clip now starts with the recommended Enhance, so its edit and export differ from the flags-off build's by design, as well as by the IndexedDB version. P209's dev hook sets `Enhance::v5()` for case 1, as planned.

**Tests (+17).**
- `set-enhance.test.ts` 9 (a new clip's start, a bad stored value, flags off, Motion, Voice, undo and redo, no change, a failed swap, Back to recommended).
- `run-pipeline.test.ts` +3 (`lvl48` written beside `out48`; a clip starting on Level previews its program; a run again keeps the clip's own).
- `start-export.test.ts` +3 (flags off: V6.1's keys and `prog48.f32`; the item counts; `enhance_off` and the Level program).
- `analytics.rs` +1, `server/src/analytics` +1.

**Tier 3** (D1):
- Rust **1144 / 0 / 5** (`cargo test --workspace`, `.env` loaded; `offcut-api` 335 again after the clippy fix);
- Vitest **764**; `pnpm e2e` 59 passed (keyed `build:e2e`);
- `pipeline-preview.spec.ts` (media, flags off) 17 passed: the clip's folder still holds exactly `out48.f32`, `prog48.f32`, `source`;
- `pnpm check` exit 0.

**Open.** 35, 39. Known issues 5, 8, 10, 11 (P212, P210, P209, watch). Not pushed (push 4 is P209). For the founder: the panel's place (Style tab) and the two settings lines, with a capture in `~/v6.2-review/look-fixes/p208-enhance-panel-1280x800.png`. Next: P209 (Enhance end to end, DC-16; push 4).

## Entry 75 - 2026-11-13 - P209: Enhance end to end, DC-16; push 4

**The founder's answers (2026-11-13).** The Enhance panel's place, under the Style tab's cards, and the two settings lines of P208 are approved for now; the founder will ask for changes after using them.

**What changed.**
- **DC-16, in the page.** `set-enhance.ts`' `withEnhance` and `isV5`: an Enhance equal to `Enhance::v5()` leaves `EditState` without `enhance`, at a clip's start (`run-pipeline.ts`) and on a switch. Rust already leaves it out of its JSON (D-485). Before this, a flags-on clip on V5's settings would have stored and hashed `"enhance":{…}`, unlike V6.1.
- **The video mirror** (development and test builds only, as the caption mirror):
  - `Scene::video_crop` and `video::Track::crop`;
  - `RenderSession::video_mirror(t)` answering `{ src, crop }` (`scene_api.rs`, additive);
  - the render worker's `videoMirror` (`atMs` in, the worker makes it an `OutMs`), `load-render.ts`' type, `dev-faults.ts`' `videoMirror`.
- **Three dev hooks:**
  - `startAsV5()`: the next clip is made as V5 made it, the AI plan switch off and `Enhance::v5()` as its start (`dev-faults.ts`);
  - `videoMirror(at)`;
  - `programSamples(at)`, the preview's program samples.
- **`flags-identity.spec.ts`** (D-440 (8)):
  - case 1's flags-on clip is made through `startAsV5()`;
  - the version pin reads 3 with `ai_plan` (the `plans` store, D-469), else 2;
  - the flags-off run is made once for both cases;
  - **case 2** is new: with the defaults, the clip record, the transcript, the voice (audio packets) and the frames' count and size are V5's; the edit differs by `enhance` (`{ clean, motion: true }`) alone, and so the cache key; the mirror shows V5's crop at every time read and the drift moving `src` off it at some.
  - **Known issue 10 closed.**
- **`enhance-panel.spec.ts`** (new), one clip and a second:
  - `lvl48.f32` beside `out48.f32`;
  - the recommended start;
  - Motion off: every time read shows the crop again;
  - Voice off: the preview's program is `lvl48.f32`'s, sample for sample at 31 places, and `prog48.f32`'s again after undo; redo brings `lvl48` back;
  - each switch within the edit budget, by the page's own measure;
  - the memory `{ level, motion: false }`;
  - the next clip starts with both off, previewing its `lvl48`;
  - settings' "Back to recommended" says so and forgets the memory.
- **`FLAG_SUITES`** gains `enhance-panel.spec.ts`. ~~so CI's media job runs it~~ **Corrected in P210 (Entry 76):** CI runs the flags suites with `--grep @no-gpu` only, and this spec has no such case, so it runs on D1 only, as `flags-identity.spec.ts` does. `openClipSignedIn` gains an optional `before` step.
- **Docs:** TS §5 row; "As read (P209)" under D-440 ((8), (24)).

**Both on `dist-flags/`, twice each** (D1, keyed builds, `--project=media-flags`, two separate runs):
- 5 passed and 5 passed: `enhance-panel` 2.0 min, `flags-identity` case 1 2.9 min, case 2 1.5 min;
- the switches: Motion 15 ms and 15 ms, Voice 74 ms and 67 ms.

**Declared.**
- D-440 (8) as above.
- D-440 (24): `set-enhance.test.ts`' two P208 cases that read `{ clean, motion: false }` in the edit now read no `enhance` (the normalisation), every other assertion kept; two cases are added for it.

**Tests (+5).** `scene_api.rs` +1 (the mirror: the crop with motion off, the drift with it on); `set-enhance.test.ts` +2; `flags-identity.spec.ts` +1; `enhance-panel.spec.ts` 1.

**Tier 3** (D1):
- Rust **1145 / 0 / 5** (`cargo test --workspace`, `.env` loaded);
- Vitest **766**; `pnpm e2e` 59 passed (keyed `build:e2e`);
- the two flags suites as above;
- `pnpm check` exit 0.

**Push 4** (not waited on): P206a to P209, read at P210's start. Known issue 11 (the lavapipe SIGSEGV) applies: one re-run of the failed job if it shows.

**Open.** 35, 39. Known issues 5, 8, 11 (P212, P210, watch). Next: P210.

## Entry 76 - 2026-11-13 - P210: the AI plan end to end

**Push 4's CI** (run 43412796622 on `0dd99db`, P206a to P209): attempt 1 failed only on the lavapipe SIGSEGV in `golden_frames` (known issue 11); one re-run of the failed jobs passed everything: 33 goldens (23 at 0.9997, 3 at 0.9999, llvmpipe LLVM 21.1.8), Vitest 766, Playwright 59, the media cases 25 passed with 1 flaky (the first-run model download, known issue 3) and 1 skipped, the flags cases all passed with 1 skipped. Known issue 11 has now shown on 3 of V6.2's 4 pushes (1, 3, 4), each time cleared by one re-run.

**A correction to Entry 75.** It said `FLAG_SUITES` gaining `enhance-panel.spec.ts` makes CI's media job run it. It does not: CI runs the flags suites with `--grep @no-gpu`, and neither `enhance-panel.spec.ts` nor `flags-identity.spec.ts` has such a case (the run's log names neither). Both run on D1 only, as planned for their acceptance. Entry 75's line is struck through and points here.

**What changed.**
- **`fake-api.ts`' `plan` option** (D-444): absent by default, so `/plan` still answers 404 and every existing suite is unchanged. With it, a request without the fake's bearer is 401 (as the server, D-494), and one with it gets `response` after `delayMs`, or `failure`. `api.setPlan` changes it later (for "Regenerate plan").
- **`web/tests-e2e/fixtures/plan/speech_60s_portrait.json`**, a model's `PlanResponse` for the reference clip's cloud transcript. It was made by hand from TE-15's DeepSeek Flash answers for `a01-scriptA` on the items of P206's `all_kinds.rs`, whose word indexes are the cloud transcript's: the hook ("I almost quit making videos", accent "making"), two emoji, four emphases (one a punch word), two zooms and two icons. That is 11 items, and the browser's validation keeps all of them. It sits beside `fixtures/cloud/`, where the e2e fixtures live. The build guide's "a plan from P211's first run if available" was not available: P211 comes after.
- **`helpers/plan-fixtures.ts`:** the cloud answer with three marker words planted at words no item points at (`Zorblax`, `Quintessa`, `Vexmoor`); the recorded plan and its kinds; the regenerated plan (3 of its items, a new post caption); and `signedInPlan()`.
- **`ai-plan.spec.ts`** (new, in `FLAG_SUITES`), 11 cases:
  - *Signed in, the switch on, one clip transcribed in the cloud:*
    1. One `POST /plan`, its words the transcript as heard, markers included. `plan_result` is `{ model, ok, items 11, dropped_by_reason {} }`. No fallback line. Only `POST /api/v1/plan` carries a marker word, and no request carries any of 20 pieces of 32 bytes of the clip. The list shows the model's 11 items in time order, "11 of 11 visuals on", the text for the post and "Regenerate plan". axe 0.
    2. Each Copy puts its text on the clipboard.
    3. At the emoji's word ("nobody") the caption shows it, and the pointer finds a caption word with the emoji drawn over the block.
    4. Each of the 11 items switched off (10 of 11), then undone (11 of 11).
    5. The export says "4 words emphasized" and "7 visual moments", and `export_done` carries `items_kept: 11, items_disabled: 0`.
    6. Run again: a new clip whose scene is refused once after its plan was kept (`failNextSetScene`); "Try again" makes no request, shows the same 11 items and sends no second `plan_result`.
    7. "Regenerate plan": one request, the 3-item plan and its new caption; then a 429 `plan_quota` makes a second request, says "…this month's AI plans are used…" and changes nothing. axe 0.
  - *Signed out* (transcribed on the device): no `/plan` request, only V4's kinds, the sign-in line once, and `plan_result` `{ rules, signed_out }`. No request carries the media.
  - *The switch off* (Settings, then the clip): no `/plan` request, the rules plan, no "Regenerate plan", no fallback line, `plan_result` `{ rules, off }`. No request carries a marker word or the media.
  - *Fallbacks:* 502 `plan_unavailable` and 429 `plan_quota`. Each makes one request, gives the rules plan and no text for the post, and shows its line once.
- **Read in the first run:** case 6 first used `failClip` on the ready clip. A ready clip forced to `failed` keeps its files open, so its run again failed `E_STORAGE_IO` ("Offcut could not save on this device"). That is a fault of the hook, which skips the clip machine on purpose, not of a real failure. The case now uses a failure during processing.
- **Known issue 8** ("Planning the edit" not shown) moves to P211: the fake answers at once, so the wait can only be read on the real API, and the founder judges it at CP 4.
- **Docs:** TS §5 rows for the spec, the helper and `fixtures/plan/`. "As read (P210)" under D-444, D-468, §13.4's row and §14.1.

**`ai-plan.spec.ts` twice on `dist-flags/`** (D1, keyed `build:e2e:flags`, `--project=media-flags`):
- Run 1 (spec alone): 9 of the 11 cases passed. Case 6 failed as read above, and case 7 did not run after it. After the fix, the signed-in group's 7 cases passed (44.5 s with setup).
- Run 2, inside the whole of `FLAG_SUITES`: all 11 passed. Signed-in cases 5.8 s, 0.2 s, 0.7 s, 2.1 s, 17.6 s (export), 5.7 s, 0.9 s. Signed out 1.1 min (device transcription). Switch off 5.1 s; 502 5.7 s; 429 4.7 s.

**V6.1's flags suites in `dist-flags/`** (D-444), the same run: 47 passed and 1 skipped (`rejections.spec.ts`' standing skip), in 13.5 min. That covers route-long 9, rejections 13, long-resume 6, high-res 4, flags-identity 2 and enhance-panel 1, besides `ai-plan`'s 11 and setup's 2. The Enhance switches took 17 ms (Motion) and 81 ms (Voice).

**Declared.** No existing test case changed. `fake-api.ts` only gains an option and a method. `FLAG_SUITES` gains one name.

**Tests (+11).** `ai-plan.spec.ts` 11.

**Tier 3** (D1):
- Rust **1145 / 0 / 5** (`cargo test --workspace`, `.env` loaded; no Rust changed);
- Vitest **766**; `pnpm e2e` **59** passed on a rebuilt keyed `build:e2e` (`dist/` unchanged);
- the flags suites as above;
- `pnpm check` exit 0; the gate `gate: ok`.

**Not pushed** (push 5 is P212).

**Open.** 35, 39. Known issues 5, 8, 11 (P212, P211, watch). Next: P211 (the real product, CP 4; five real calls, about $0.01, at the founder's word).

## Entry 77 - 2026-11-13 - P211: the real product, checkpoint 4 (readings)

**The real product on D1, read for speed, cost and look** (TE-15's product read, E-11; both in `docs/v6.2/experiments.md`). The setup:
- the local API with the real DeepSeek key, `PLAN_ENABLED=true`, `deepseek-flash`;
- `dist-flags/` behind a scratch proxy, so the app and the API share one origin;
- transcription on the device for the agent's runs;
- signed in as a local test address through a magic link written into the local database (no mail sent);
- the reference clip five times, from a throwaway Playwright driver (deleted, never committed).

**Every call** (six, listed one by one in `experiments.md`):
- **What happened:** all six answered 200 on the first try, with no retry, no item skipped by the parse, a hook title in every plan and no fallback.
- **Speed:** 4.0 to 5.3 s in the browser.
- **Size:** 7,433 input tokens each (7,296 cached after the first) and 902 to 1,167 output tokens.
- **Cost:** the server's spend counter says 21,146 µ$ (about $0.0035 a plan, cached input counted at the full price); DeepSeek's balance went $1.80 → $1.79.
- **The extra call:** the first attempt of the driver hung on its own bug after one call (a locator read with no timeout), so it was stopped and run again; that call is call 0.

**Drops** (10 of 63 items, median 14.6%, over TE-15's 10% line):
- `zone_taken` 5, an icon placed on a word inside the list's span;
- `number_unresolved` 4, the device transcript writes "$12 ,000";
- `anchor_rule` 1.

**E-11:** `plan_result` and `export_done` arrive in the local sink with every V6.2 field (`items_kept`, `items_disabled`, `enhance_off ["voice", "motion"]` with both off) and no content. Pass.

**Captures and exports** are in `~/v6.2-review/product/`, with a README for the founder: the two exports (the defaults; Voice cleanup and Motion off), the editor, settings, and each item of run 1 at its time.

**Four findings, put to the founder:**
1. The Visuals tab lists items the validation left out, with their switch on ("10 of 10 visuals on" with 8 drawn), and `items_kept` counts them.
2. The Hook title row's description is the whole transcript (its words are the clip's).
3. "What we changed" says "Voice cleaned" with Voice cleanup off (`summary.rs` always said true).
4. The drop rate above.

**A mistake of the agent's.** The local stack was left for the founder with cloud transcription off (`TRANSCRIBE_ENABLED=false`, set for the agent's runs), and the message did not say so. The founder found no Cloud option. The API was restarted with it on (17:15 UTC). A stack handed over for a checkpoint now runs the product's own settings, and the message says what differs.

**The founder's answers (CP 4, 2026-11-13).**
- **Fix 1, 2 and 3:** P211a.
- **The hook title:** fix 2 only, the row; nothing else about the hook.
- **The post caption:** shorter: one or two sentences, at most 150 characters, asked in the prompt; the 2,200-character check stays. The founder had read it as a caption for Instagram and other social sites. P211b, with a re-read of about $0.02.
- **The landing page:** "Start free" becomes "Start creating". P211a.
- **Finding 4:** not asked for.
- **Still to come after P211a and P211b:** the go, and the founder's read of known issue 8.

**Open.** 35, 39. Known issues 5, 8 (the founder's read at CP 4), 11. Next: P211a, then P211b, then the founder's go.

## Entry 78 - 2026-11-13 - P211a: the founder's CP 4 fixes

**Asked for at CP 4** (Entry 77): fixes 1, 2 and 3, and "Start creating" on the landing page. Recorded as D-504 to D-507 in the plan.

**What changed.**
- **Fix 1, the list shows what is drawn (D-504):**
  - `set_scene` now also answers `kept`, the ids of the plan's items its validation kept. It reads its input a second time and validates it once more, so `scene_of` and the two tests that build `SceneReport` are unchanged; `KeptReport` flattens `SceneReport` and adds the field.
  - Every caller of `setScene` stores the ids (`clip-store`'s `planKept`, `setPlanKept`): the pipeline, the three review updates and the preview's reload.
  - `listedItems` is the one rule: with `ai_plan` on, only the kept items; with it off, or before a scene says, every item, as in V6.1. The Visuals tab uses it, and so does `export_done`'s `items_kept` / `items_disabled`.
- **Fix 2, the hook row (D-506):** the Hook title row's description is the hook's own text (the plan's validated `extras.hook`), no longer its words, which are the whole clip.
- **Fix 3, "Voice cleaned" (D-505):** `Scene` keeps whether the edit's voice is `Clean`, and `summary()` reports that as `voice_cleaned`. It is true for every V5 and V6.1 edit, so their summaries are unchanged; with Voice cleanup off (`Level`) "What we changed" no longer says "Voice cleaned".
- **The landing page (D-507):** `landing.cta.startFree` reads "Start creating" in the top bar, the hero, pricing and the final call. The key keeps its name; two comments that quoted the old words are updated.
- **The gate:** two ALLOW patterns for the lines this changes, `protocol.ts`' one-line `SceneReport` (now a block with an optional `kept`) and `scene_api.rs`' `to_js(&SceneReport { dropped, chunks })`.

**Tests (+4).**
- `scene_api.rs` +1: the ids kept, as many as `validate_plan` keeps with a word edit, none for a refused plan, and the answer's shape.
- `offcut-scene`'s `lib.rs` +1: `voice_cleaned` false with `Level`, true with `Clean` and with `Enhance::v5()`, the items counted the same.
- `start-export.test.ts` +1: an item the scene left out is not in `items_kept`; `listedItems`' three cases.
- `ai-plan.spec.ts` +1: a plan with one more item that the validation leaves out (an emoji past the transcript's end). The list shows the 11 kept, "11 of 11 visuals on"; `plan_result` counts the drop; the Hook title row reads "I almost quit making videos" and none of the transcript.

**Declared.** No existing test case changed. `SceneReport`'s `kept` is optional in `protocol.ts`, so the Vitest fakes that answer `{ dropped, chunks }` are unchanged.

**Browser runs** (D1, keyed builds):
- **`dist-flags/`:** `ai-plan.spec.ts` 12 of 12, the new case in 4.9 s; `enhance-panel.spec.ts` 1 of 1 (Motion 18 ms, Voice 57 ms, inside the 150 ms budget with the second validation); `flags-identity.spec.ts` 2 of 2.
- **Read along the way:** `flags-identity`'s case 1 failed once, because the run had a fresh `dist-flags/` and a stale `dist/`. The render-cache key holds `RENDERER_VERSION`, a hash of the render WASM, so two builds of different WASM give two keys by design. It passed on the two builds made from the same tree.
- **`dist/`:** `pnpm e2e` 59 passed (the landing page with "Start creating"); `review-edit.spec.ts` and `export-free.spec.ts`, 30 passed with setup (the flags-off list and summary unchanged).

**Tier 3** (D1):
- Rust **1147 / 0 / 5** (`cargo test --workspace`, `.env` loaded);
- Vitest **767**;
- the browser runs as above;
- `pnpm check` exit 0; the gate `gate: ok`.

**The local CP 4 stack** serves the rebuilt `dist-flags/`, so the founder sees these fixes on a reload. Its API now runs from a copy of the binary in the scratchpad, so the workspace tests could rebuild `target/debug/offcut-api.exe` (Windows locks a running file); cloud transcription is on.

**Not pushed** (push 5 is P212).

**Open.** 35, 39. Known issues 5, 8 (the founder's read at CP 4), 11. Next: P211b (the post caption at most 150 characters; a re-read of about $0.02), then the founder's go.

## Entry 79 - 2026-11-14 - P211b: a shorter post caption

**Asked for at CP 4** (Entry 77): a shorter post caption, one or two sentences of at most 150 characters, to post on Instagram and other social sites. Recorded as D-508.

**What changed.**
- **The prompt:**
  - `prompt.rs`' extras line asks for the post caption in "one or two sentences and at most 150 characters" (`POST_CAPTION_TARGET_CHARS`, new);
  - the output schema's `maxLength` and the validation's `POST_CAPTION_MAX_CHARS` stay 2,200, so a longer answer is still kept, not dropped;
  - the system message grows from 16,500 to 16,524 characters.
- **`system_prompt@v2.snap`:** that one line changes ("one or two sentences and", 2200 → 150). It was new in P186, so it is not a snapshot of the gate's base.
- **The recorded request:** `request.recorded.json` holds the new length, 16,524, from this re-read's record, and `deepseek_adapter.rs`' comment says so; every other field is as recorded before.

**The re-read** (TE-15, `experiments.md`; the founder's yes at CP 4):
- **Calls:** 16 Flash calls on the 15 transcripts (one parse retry), $0.0171.
- **Captions:** every one 79 to 148 characters (median 103), one or two sentences, all 15 kept by the validation.
- **The hook:** kept 15 of 15.
- **Drops:** median 12.5% (11.1% at P206c's second re-read). The reasons moved between kinds whose paragraphs did not change; read as variance.

The annotated plans are in `~/v6.2-review/p211b/plans/`.

**Declared.** `deepseek_adapter.rs`' recorded length (16,500 → 16,524) and its comment, as in P206c (Entries 71, 72). No assertion changed.

**Tests.** `offcut-plan` and `offcut-api` 415 passed, 0 failed (`.env` loaded); clippy clean.

**The local CP 4 stack** gets a rebuilt API with this prompt, so the founder's runs show the short caption.

**Not pushed** (push 5 is P212).

**Open.** 35, 39. Known issues 5, 8 (the founder's read at CP 4), 11. Next: the founder's go at CP 4, then P212.

## Entry 80 - 2026-11-14 - P211: the founder's go at checkpoint 4

**The founder's go (2026-11-14):** "go i have verified everything". They ran it on the local stack (the rebuilt `dist-flags/` with P211a's fixes, and the API with P211b's prompt and cloud transcription on).

**P211's acceptance:**
- the readings recorded, every call listed (Entry 77, `experiments.md`);
- the founder's go recorded (here);
- the commits `371c935`, `ddf5a92` (P211a) and `d2be873` (P211b).

**Known issue 8** ("Planning the edit" not shown; the plan's 4 to 5 s wait inside "Finding visual moments") had no separate word from the founder, so it stays open.

**Open.** 35, 39. Known issues 5, 8, 11. Next: P212 (copy-back, the full runs, push 5, merge, tag).
