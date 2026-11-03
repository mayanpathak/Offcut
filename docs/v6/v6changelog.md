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
| 6 | A 10-minute cap, if TE-18 or TE-20 fails on real laptops | Founder | PS §20.2A |
| 7 | The launch date, when V6.11 closes | Founder | PS A-2 |
| 8 | The long-path fixtures and labelled set of at least 10 real long videos | Founder, by V6.4 | BP §8A.4 |
| 10 | Dodo Payments' verification: under review as an individual, stated as up to 72 hours. Read TE-9's cases in test mode | Founder | Entry 0.1, TE-9 |
| 14 | Tell Dodo the pricing and legal pages are live, and check its product form's description against them | Founder | Entry 3 |
| 15 | Before V6's merge: Vercel blocks a production deploy from this repository while it is private (Entry 5). Stay public through merges, or test merging with the Gmail address (branch `vercel-email-check`) | Founder | Entry 5 |
| 16 | Delete the webhook logger on Render, and its endpoint in Dodo's test mode. The endpoint for the deployed site is registered at G 14.6 | Founder | Entry 13 |
| 17 | Read the legal and pricing words P148 changed (Entry 17), before the merge puts them live. With them: from the merge until the founding release, the account page's two offers lead to Dodo's test mode (D-285), where the pages say Creator is not on sale. Keep the offers, or hide them on the live site until then | Founder, before P150 | Entry 17 |

## Known issues for later prompts

| # | Issue | Prompt |
|---|---|---|
| 3 | Carried from V5 (its known issue 3): `openEditor`'s wait for the drop zone fails on the hosted runner, most often on the first model case after `cancel.spec.ts`. Run 41502704869 failed it and its retry; a re-run passed. Its message showed `UNSUPPORTED_STORAGE`, so the storage check now has a second try (Entry 12). Whether that cures the flake is read on the next CI runs: the first attempt's page held the drop zone's words only as the 90 s ended, which a storage retry may not explain | Next CI runs |

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
