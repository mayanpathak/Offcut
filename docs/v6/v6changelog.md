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
| 9 | Read the 3 template mails sent to the second Gmail address on 2026-11-02 17:22 UTC: inbox or spam, the visible From, and that the link's host is `example.invalid` (D-255). Delete the Render mail-probe service, which holds the SMTP key | Founder | Entry 7 |
| 10 | Dodo Payments' verification: under review as an individual, stated as up to 72 hours. Read TE-9's cases in test mode | Founder | Entry 0.1, TE-9 |
| 14 | Tell Dodo the pricing and legal pages are live, and check its product form's description against them | Founder | Entry 3 |
| 15 | Before V6's merge: Vercel blocks a production deploy from this repository while it is private (Entry 5). Stay public through merges, or test merging with the Gmail address (branch `vercel-email-check`) | Founder | Entry 5 |

## Known issues for later prompts

| # | Issue | Prompt |
|---|---|---|
| 3 | Carried from V5 (its known issue 3): `openEditor`'s wait for the drop zone fails on the hosted runner and passes on the retry. Cause not proven. Its failure now says what the page showed | The first CI run that shows it |

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
