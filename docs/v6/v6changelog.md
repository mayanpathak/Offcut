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
| 21 | The Submagic benchmark shorts and the caption references, outside git | Founder | Entry 21 |
| 22 | The local Tooscut copy (`sh2clips/old_project_study_editor/`) holds no licence file; keep upstream's licence text beside it so the study rule's basis is on disk | Founder | Entry 21, Entry 0 |

## Known issues for later prompts

| # | Issue | Prompt |
|---|---|---|
| 3 | Carried from V5 (its known issue 3): `openEditor`'s wait for the drop zone fails on the hosted runner, most often on the first model case after `cancel.spec.ts`. Run 41502704869 failed it and its retry; a re-run passed. Its message showed `UNSUPPORTED_STORAGE`, so the storage check now has a second try (Entry 12). Whether that cures the flake is read on the next CI runs: the first attempt's page held the drop zone's words only as the 90 s ended, which a storage retry may not explain. Push 2's run 41602738386 and push 3's run 41822657804: nothing flaky. **The merge's run 41827888094 failed it on both tries** (`model-download.spec.ts`'s first case, right after `cancel.spec.ts`): the first try held "Checking your browser…" for 90 s, the retry read `UNSUPPORTED_WEBGPU`, and the setup's own check read `UNSUPPORTED_WEBGPU` once that run. A re-run of the job passed. Suspects: the runner's WebGPU adapter coming and going, and `start-app.ts`'s `anonymousId()`, an IndexedDB read with no timeout before the capability check (since V1) | V6.1 |

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
