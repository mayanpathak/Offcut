# v6implementation.md - Offcut V6 "Accounts, limits, payments": implementation plan

**Implements:** `buildplan.md` section 8 (V6), which is steps M1.7 and M1.8 of `technicalspec.md` (TS) §28, under the compatibility contract of TS §41, with the seam S-10 of TS §40.9 and the comment cleanup DC-11. Product references are to `product.md` (PS). "BP §n" is a section of `buildplan.md`; "V5 §n" of `docs/v5/v5implementation.md`; "V5 CL" is `docs/v5/v5changelog.md`; "V6 CL" is `docs/v6/v6changelog.md`. Decisions D-1 to D-240 are V1's to V5's; this document continues at D-241.

**Reader.** Whoever writes the V6 code. `docs/v6/v6buildguide.md` gives the order of work and the commands; `docs/v6/coding-promptsv6.md` gives the prompts an agent runs. This file gives the contracts and the decisions, and wins where the three disagree. It was written after reading the code at `3cae48e` (the tree of `origin/main` at `4ff3b74` plus one docs commit) on 2026-11-02; every "today" below is a fact of that commit. Each decision the specs leave open is marked **(V6 decision)**. A change to something an earlier version froze is marked **REOPENED CONTRACT**. A number nobody has measured is marked **(assumption)**. A fact of a third party that its documents do not settle is marked **(read in test mode)**.

**The one-sentence goal.** At the end of V6 a stranger can drop a clip, see the limits before signing in, sign in from an emailed link, export a 720p watermarked file, see "2 of 3 left", pay for Creator, and export 1080p with no watermark; can delete their account or take their data with them; and nothing about the clip path has changed.

**What you can demo (BP §8).** J1 to J12 with a real email address and the merchant's test mode.

**The second goal: V6.1 to V6.11 stay additive (TS §41.5).** V6 leaves accounts, the entitlement token and one list of per-user tables, so V6.2's `ai_usage` and `ai_shorts_grants` are two more entries (S-10). Nothing else is reserved.

---

## 0. Contents

1. Scope. 1A. Preconditions. 2. Decisions (D-241 to D-292). 3. Dependencies and environment. 4. File tree. 5. Build order. 6. Mail. 7. Sign-in and sessions. 8. Entitlement and receipts. 9. Account deletion and export. 10. Billing and the provider. 11. The client's foundations. 12. The gate and the export. 13. Interface and copy. 14. Analytics. 15. Tests. 16. CI and deploy. 17. Experiments. 18. Risks. 19. Hand-over to V6.1. 20. Exit checklist. 21. Copy-back to the specs.

---

## 1. Scope

### 1.1 In V6

| Area | What works at the end of V6 |
|---|---|
| Cleanup | The old V6.x numbers in code comments are rewritten through TS §42, proven comment-only (DC-11). Known issues 10 (`aria-details`) and 3 (`openEditor` flake) are addressed |
| Mail | `lettre` over SMTP through Brevo, on a port the Render free service can reach, behind the `Mailer` trait. One template |
| Auth server | Magic link, verify, refresh with rotation and reuse revocation, logout, `GET /me` |
| Entitlement server | `issue()`, `GET /entitlement`, `POST /usage/receipts` |
| Account server | `POST /account/delete`, `GET /account/export`, the per-user table list (S-10), retention of `magic_links`, `sessions`, `webhook_events` |
| Billing server | `BillingProvider`, the provider's adapter, `offers.rs`, webhook verification, dedupe, transition table, `POST /billing/checkout`, `GET /billing/portal`, `POST /billing/webhook` |
| Client | Auth machine and store, entitlement store, `effective()`, receipt outbox, broadcast between tabs, session restore |
| Interface | `SignInDialog`, `AuthCallbackPage`, `AccountPage`, `UpgradePrompt`, the counter on `ExportButton`, the last-export confirmation, copy for 8 errors and 3 blockers |
| Gate | The last three blockers; a real token from the account; a receipt on a new render only |
| Analytics | `signin_step`, `upgrade_prompt`, `checkout_step` start. All 19 events now fire |
| Experiments | TE-8 (mail), TE-9 (merchant) |

### 1.2 Not in V6

| Left out | Arrives |
|---|---|
| Passwords, OAuth, Pro, coupons, credits, server-enforced export limits | Never in the MVP (TS §36) |
| Live-mode payments for strangers | The founder switches the keys when verification is approved and the launch nears (D-285) |
| `features.ts` and any feature flag | V6.1 (TS §41.3). V6 is not flagged: accounts replace "accounts are not open yet" at the merge |
| `api-cold-start.spec.ts`, `restoreClip` | V7 |
| `privacy-network.spec.ts`, Privacy Policy text | V8 |
| Founding offer on (`FOUNDING_OFFER_ENABLED=true`) | V9 |
| Long video quota, AI quotas | V6.2, V6.4 |

### 1.3 Journeys and use-case steps served

| Flow | Served in V6 |
|---|---|
| J9 Export click, C-7 Gate, C-8 Sign-in | Full |
| J11 C-10 Receipt | The receipt half (the rest since V5) |
| J12 C-11 Upgrade and checkout | Full |
| C-15 Cold start | Sign-in, checkout and account pages; the `API_WAKING` line |
| C-18 Delete, export data; C-19 Webhook | Full |

---

## 1A. Preconditions

- [x] 1. **V5 is closed.** `v5` is tagged; `origin/main` is `4ff3b74`, deployed.
- [x] 2. **The merchant is applied for.** Founder's dashboard on 2026-11-02: personal information approved, product and bank details recorded, review stated as up to 72 hours, test mode available.
- [x] 3. **A mail account exists.** Brevo; five `SMTP_*`/`MAIL_FROM` values in `.env.deploy`; login on port 2525 succeeds from D1; one test mail accepted (V6 CL Entry 0.1).
- [x] 4. **The provider is Dodo Payments.** Founder, 2026-11-02.
- [x] 5. **Mail test 1 arrived.** Founder, 2026-11-02. Whether it reached the inbox or spam is not recorded yet (V6 CL open item 12). It decides nothing; TE-8 does.
- [x] 6. **The port probe was read on Render** (D-253, §6.2): 2525 open, 587 and 465 blocked.
- [x] 7. **The founder said yes to DC-12, "carefully"** (D-246). The row is in TS §41.2.
- [ ] 8. **A draft pull request `v6-build` → `main` is open** after Setup. CI runs on pull requests only.

---

## 2. Decisions this document makes

All are **(V6 decision)** unless they say otherwise.

### 2.1 Process

| # | Decision | Why |
|---|---|---|
| D-241 | **All of V6 is on `v6-build`**, made from `v5-build` at `3cae48e`. Commits are titled `V6 P<nn>: <title>`. The agent commits and pushes there. Opening or merging the pull request, pushing to `main`, tagging, dashboards and production reads are the founder's | The standing rule. `3cae48e` is a docs commit that reaches `main` with this merge. The v1.3 specs, uncommitted today, are this branch's first commit |
| D-242 | **V5's speed rules hold** (D-196, D-197, D-200): tiered gates, **one** full media run, in P150. Three pushes: P142 (not waited on), P148 (not waited on), P150 (waited on) | The founder's standing preference |
| D-243 | **Compact documents.** One plan, one guide, one prompts file; short changelog entries | D-195 |
| D-244 | **The floor is V5's close** (TS §41.1): Rust 657 / 0 / 4; Vitest 336; `pnpm e2e` 12; verifier self-test 16; media 94 + 1 skip. V6 reads no baseline again beyond Setup's quick counts | No code changed since |
| D-245 | **DC-11 runs one pass over one map**, never one number at a time (§5.2). A script in the scratchpad, not committed, applies V6.1→V6.2, V6.2→V6.3, V6.3→V6.7, V6.4→V6.6, V6.5→V6.6, V6.6→V6.6, V6.7→V6.10 in a single regex substitution. Today's count is 49 lines across `crates`, `server`, `web/src`, `verify`, `scripts`, `.github`; TS §42 says about 57 | Rewriting V6.1→V6.2 and then V6.2→V6.3 would move every first hit twice. Prove: the diff with comments stripped is empty |
| D-246 | **New row DC-12 of TS §41.2 (founder's yes of 2026-11-02, "carefully").** In `export-creator.spec.ts`, case "no token" (lines 125-136 today): the title, the assertion of line 127, which becomes "`SignInDialog` is visible with the limits", and one new line that closes the dialog, as the next case in this serial group needs the page free. **Lines 128-135 stay as they are:** no progress, no download after 11 s, no `export_started`, nothing in `exports`, the button enabled. `messages.export.unavailable`, `setUnavailable` and the store's `unavailable` go. Every other case stays. P147 reads the spec's diff and reports it line by line. Any further change to this file is a stop | It is the one protected assertion V6 must change: that sentence stops being true by design. What the case protects, that no token means nothing is exported, stays proven. Nothing about a file changes |
| D-247 | **D-69, D-74, D-114, D-202 hold.** Readings are on D1, labelled `d1` | Standing decision |

### 2.2 Mail

| # | Decision | Why |
|---|---|---|
| D-248 | **`lettre` over SMTP through Brevo** (founder, 2026-11-02). TS's HTTP mailer is replaced. `server/src/mail/http_mailer.rs` becomes `smtp_mailer.rs` | The founder's choice. Brevo verifies one sender with no domain, 300 mails a day |
| D-249 | **Features:** `default-features = false`, `smtp-transport`, `builder`, `hostname`, `tokio1-rustls`, `ring`, `webpki-roots`. Compiled and connected with STARTTLS to Brevo on 2525 from a scratch project | Defaults bring `native-tls`, where the server uses rustls; no `tokio1` would block async handlers |
| D-250 | **`0BSD` joins `deny.toml`'s `allow`**, for `quoted_printable`. Read: permissive, no attribution requirement | The one licence error `cargo deny` reports |
| D-251 | **Variables:** `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `MAIL_FROM`. `MAIL_API_KEY` goes from `config.rs`, `render.yaml`, `.env` and tests | `config.rs` aborts on a missing variable, so Render holds the new values **before** the deploy that reads them (G 8.6) |
| D-252 | **Port 2525 with STARTTLS** on `smtp-relay.brevo.com`. Render's free web services block 25, 465 and 587 since 2025-09-26 | Render's changelog. 2525 is not on that list |
| D-253 | **A real read from Render comes first** (§6.2): a throwaway free service on a public Python image, holding no secret, reports which ports open, and is deleted. If 2525 is blocked: **stop and ask** (TS §41.4). Recommendation then: Brevo's HTTP API behind the same trait, without `lettre` | Nothing else can prove it, and it touches no production service |
| D-254 | **The mail is sent inside the request, with an 8 s timeout.** Failure is 503 `email_unavailable` | C-8 needs `E_AUTH_EMAIL_UNAVAILABLE` in the dialog. The client's attempt timeout is 10 s |
| D-255 | **Plain text and a small HTML part, one link, no image, no tracking.** TE-8 reads a delivered mail: the link's host must be `APP_ORIGIN`. If Brevo rewrites it, turn its tracking off; if that cannot be done, stop and ask | A rewritten link would hand the sign-in token to a third host |
| D-256 | **The Gmail sender is a known risk.** Brevo marks it "Freemail domain is not recommended". TE-8 decides. Failing it is a stop-and-ask: recommendation, a domain at about 10-15 USD a year, which the founder's rule (BP §1.1) reserves to the founder | A link in spam is a failed sign-in |

### 2.3 Auth

| # | Decision | Why |
|---|---|---|
| D-257 | **`getrandom` 0.4 is a direct dependency of `offcut-api`**, added to its wrappers in `deny.toml`. It alone makes the 32 random bytes of a link and a refresh token | Two UUIDs are not 32 uniform bytes. Today `deny.toml` bans a direct use |
| D-258 | **A session has two valid hashes for 30 s, and every accepted refresh rotates** (§7.3), including one that uses the previous hash | The client's own retry can lose a response during a cold start. Answering a retry with no new cookie would leave the browser holding a token that is revoked 30 s later, signing a paying user out |
| D-259 | **Verify and magic-link are one attempt, after the API is awake** (§11.2). `completeSignIn` awaits `ready()`, then `retry: "none"` | A link is single-use. A retried verify whose first response was lost reads `link_invalid` on a good link |
| D-260 | **REOPENED TS §22.2: `users.email` holds the normalized address.** The table `magic_links` keeps only `email_normalized`, so the address as given is gone by the time the user row is made | A migration for a letter's case is not worth it. Mail is delivered either way |
| D-261 | **An access token has the entitlement token's wire form,** base64url JSON `{sub, sid, exp}`, a dot, base64url Ed25519, signed with `ACCESS_TOKEN_SIGNING_KEY`. `AuthUser` verifies it with no database read | TS §22.2. The form is in the code and tested |
| D-262 | **Deleted or revoked within 15 minutes.** `/me`, `/entitlement`, receipts and billing read the user row; none gives 401 `session_expired` | One lookup in the routes that have one anyway |
| D-263 | **`RateKey::User` and `RateKey::EmailHash` are checked in the handler** after the extractor or the normalization, through `limit(group, key)` in `rate_limit.rs`. `by_ip` stays middleware | The key is not known to a layer |
| D-264 | **Status codes:** `link_invalid` and `link_expired` 400; `unauthorized` and `session_expired` 401; CSRF failure 403 `forbidden`; `email_unavailable` 503; `billing_unavailable` 502 | TS §22.7 fixes 502. §7.5 |

### 2.4 Entitlement and receipts

| # | Decision | Why |
|---|---|---|
| D-265 | **`offcut-entitlement/src/sign.rs`, feature `sign`.** `offcut-api` turns the feature on; the wasm crates do not, checked with `cargo tree` | TS §5 |
| D-266 | **`issue(pool, key, user, now)` is one function** with one query for the subscription and one for the count. No caller computes a plan | TS §22.3 |
| D-267 | **Every outbox row records the plan it was exported under.** `{exportId, enqueuedAt, attempts, plan}`; `effective()` subtracts Free rows only. No store has a writer yet, so no migration | TS §12.1 says "minus outbox length", which would make a Creator export spend a Free count after a lapse, and would break the protected suites: `export-creator.spec.ts` runs Creator exports and then a Free one in one profile, where three unsent rows would raise the last-export dialog |
| D-268 | **A receipt without a session waits.** `flushOutbox` needs a session; with none it leaves the rows | A seeded-token test and an offline export are the same case |
| D-269 | **`fair_use_exceeded`** is one warning per user per day, held in a small in-memory set, cleared at the UTC day change | TS §22.4. A restart may log twice, which is harmless |

### 2.5 Account

| # | Decision | Why |
|---|---|---|
| D-270 | **S-10: `db/mod.rs` holds `USER_TABLES`,** the per-user tables: `sessions`, `subscriptions`, `usage_receipts`. `account_delete.rs` asserts each is empty after a delete and fails on a table with a `user_id` column that the list omits, by reading `information_schema` | V6.2 adds two names. A forgotten table fails a test |
| D-271 | **Delete order is TS §22.7.** Cancel first; 502 and nothing deleted on failure. A second delete of a gone user is 204 | Idempotent by TS §22.1 |
| D-272 | **A late webhook for a deleted user is 200 and changes nothing** (§10.3) | Deleting cancels the subscription, so the `Canceled` event always arrives after the row is gone. A foreign-key failure would be 500 and eight retries |

### 2.6 Billing

| # | Decision | Why |
|---|---|---|
| D-273 | **Dodo Payments (founder, 2026-11-02), in test mode first.** Base URL by a new variable `BILLING_API_BASE_URL`: `https://test.dodopayments.com` or `https://live.dodopayments.com` | A key and a host switch together |
| D-274 | **REOPENED TS §22.5: `parse_event(&self, headers, raw_body)`.** The provider's event id is the `webhook-id` header, which the spec's signature cannot see | Dodo follows Standard Webhooks |
| D-275 | **REOPENED TS §22.5: `BillingEvent.owner: EventOwner`,** `User(UserId)` or `Customer(String)`. `webhook.rs` resolves `Customer` through `subscriptions.provider_customer_id` | A refund event may carry no metadata **(read in test mode)**. Parsing stays pure |
| D-276 | **REOPENED TS §22.5's table: `past_due` + `Activated` → `active`,** setting `current_period_end` | Recovery from a failed renewal can arrive as `subscription.active` **(read in test mode)**. Without this row a paying user falls to Free at period end |
| D-277 | **The interval comes from `offers.rs`,** by the event's product id; the amount is never read | The server holds one map of offers. A product id no offer names is ignored, logged `billing_event_ignored` |
| D-278 | **Signature check is Standard Webhooks, written here:** strip `whsec_`, base64-decode, HMAC-SHA256 over `id.timestamp.body`, compare any `v1,` entry in constant time, refuse a timestamp more than 5 minutes off. Uses `hmac` 0.13, already in `Cargo.lock` | A small function with published test vectors. No SDK exists for Rust that we need |
| D-279 | **The adapter's mapping is read in test mode before it is written** (§10.2). Real payloads go into `server/tests/fixtures/dodo/`, scrubbed | Dodo's pages do not document whether checkout `metadata` reaches subscription webhooks, the refund payload's fields, or which event a scheduled cancel sends. Guessing here means losing paid users silently |
| D-280 | **If `metadata.user_id` does not reach subscription events: stop and ask.** Recommendation: match by the checkout's customer email through `users.email_normalized`, which is why the customer email is sent normalized | A choice with a privacy and an edge-case cost |
| D-281 | **`reqwest` becomes a dependency,** `default-features = false`, `json`, `rustls-no-provider`; `rustls` with `ring`, installed once in `main.rs`, with `install_default` also called in the test app. Connects from a scratch project | The tree stays on one crypto provider, `ring`, which `sqlx` already brings. `reqwest`'s default would add `aws-lc-rs` |
| D-282 | **`time` adds the `parsing` feature** for the event's ISO 8601 timestamp | No new crate |
| D-283 | **The webhook route has no `AuthUser` and no CSRF,** reads raw bytes, 64 kB. It is called on the Render host directly | TS §22.9 |
| D-284 | **A `Refunded` for an unknown customer, and any event with no resolvable owner: 200, logged, nothing changed** | The provider should stop retrying what can never apply |
| D-285 | **Production runs test-mode keys after the merge,** until the founder switches them. Then wipe any test-mode `subscriptions` row. Test-card Creator on an unlaunched site is harmless | V6's gate needs a test purchase on the deployed site. Switching is four variables and a webhook registration |

### 2.7 Client

| # | Decision | Why |
|---|---|---|
| D-286 | **REOPENED CONTRACT `http.ts`: `HttpRequest` gains `bearer?: string` and `csrf?: true`** | Today no request can carry either |
| D-287 | **REOPENED CONTRACT `http.ts`: a 5xx whose body is `email_unavailable` or `billing_unavailable` is an answer, not a waking API.** No retry, and its own `AppFailure`. `api-client.ts` maps `link_invalid`, `link_expired`, `session_expired` too | Today 502 and 503 read as "still waking": retried for 70 s and shown as `E_NET_TIMEOUT`, so a provider failure would look like a network failure |
| D-288 | **`authed()` in `api-client.ts`** sends with the access token, and on 401 refreshes once, single-flight, then repeats once. Refresh failing with 401 is `E_AUTH_SESSION_EXPIRED` | Several callers can meet one expired token |
| D-289 | **`restoreSession` starts at `start-app.ts`'s marker and nothing waits for it** | INV-3: import, processing, preview and review never wait on the API |
| D-290 | **Two groups of blocker.** `forExport()` keeps V5's order and appends the three. `ExportButton` is disabled only for `B_EXPORT_IN_PROGRESS` and `B_PIPELINE_NOT_READY`; the other three let the click through to open their dialog | C-7. A disabled button cannot open a sign-in dialog |
| D-291 | **Sign-out clears the entitlement token; a lost session does not** | A signed-out person must meet `B_NOT_SIGNED_IN`. An expired session keeps offline exports until `exp` (C-15) |
| D-292 | **An undecodable token counts as none.** `entitlement-store` reads claims without verifying, for display and blockers only; the worker still verifies (INV-9). `planOf` in `start-export.ts` goes in favour of the store | One reader of claims on the page |

---

## 3. Dependencies and environment

### 3.1 Rust

`Cargo.toml` (workspace) and `server/Cargo.toml`. All checked together on 2026-11-02 in a scratch project: compiles, one crypto provider (`ring`), no `native-tls`, `cargo deny` clean after D-250 and D-257.

| Crate | Version | Features | For |
|---|---|---|---|
| `lettre` | 0.11 | D-249 | Mail |
| `reqwest` (moves from dev-only) | 0.13.5 | `json`, `rustls-no-provider` | Provider API |
| `rustls` | 0.23 | `ring`, `std`, no default | One provider, installed in `main.rs` |
| `webpki-roots` | 1 | | Reqwest's trust roots **(read at P144: `rustls-no-provider` may need them set explicitly)** |
| `hmac` | 0.13 | | Webhook signature |
| `getrandom` | 0.4 | | Tokens |
| `async-trait` | 0.1 | | `dyn Mailer`, `dyn BillingProvider` (TS §22.5) |
| `time` | existing | + `parsing`, `macros` if needed | Event timestamps |
| `offcut-entitlement` | workspace | `sign` | `sign_token` |

No new npm dependency.

### 3.2 Environment

| Variable | Change | Local value | Production |
|---|---|---|---|
| `MAIL_API_KEY` | Removed | | Remove after the deploy |
| `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD` | New, required. `SMTP_PORT` parses as `u16` | Placeholders | Brevo's values, from `.env.deploy` |
| `MAIL_FROM` | Real | Placeholder | The verified sender |
| `BILLING_API_BASE_URL` | New, required, `https://` | `https://test.dodopayments.com` | The same until D-285 |
| `BILLING_API_KEY`, `BILLING_WEBHOOK_SECRET`, `BILLING_PRICE_*` | Real | Placeholders | The merchant's test-mode values; `BILLING_PRICE_*` are product ids |

`.env` goes from 15 to 19 names. `Config`'s hand-written `Debug` redacts `smtp_password`, `billing_api_key` and `billing_webhook_secret`. `TestApp` gets fakes, not values.

---

## 4. File tree

**Created, server (23).** `auth/{session,access_token,extract}.rs`; `mail/{mod,smtp_mailer,templates}.rs`; `entitlement/{mod,issue}.rs`; `usage/mod.rs`; `account/{delete,export}.rs`; `billing/{mod,provider,adapter,offers,webhook}.rs`; `db/{users,magic_links,sessions,subscriptions,webhook_events,usage_receipts}.rs`.

**Changed, server.** `auth/{mod,magic_link}.rs`, `account/mod.rs`, `analytics/retention.rs`, `config.rs`, `db/mod.rs`, `error.rs`, `lib.rs`, `main.rs`, `rate_limit.rs`, `router.rs`, `state.rs`, `.sqlx/`, `tests/common/mod.rs`, `Cargo.toml`, `render.yaml`, `deny.toml`.

**Created, crates.** `offcut-entitlement/src/sign.rs`, `tests/{token_roundtrip,profile}.rs`.

**Created, web (20).** `state/{auth-store,entitlement-store}.ts`, `state/machines/auth-machine.ts`; `entitlement/entitlement.ts`; `persistence/receipt-outbox.ts`; `platform/broadcast.ts`; `config/pricing.ts`; `usecases/{request-magic-link,complete-sign-in,sign-out,restore-session,start-checkout,confirm-checkout,open-billing-portal,delete-account,export-account-data}.ts`; `ui/components/{SignInDialog,UpgradePrompt}.tsx`; `ui/pages/{AuthCallbackPage,AccountPage}.tsx`.

**Changed, web.** `net/{http,api-client}.ts`, `usecases/{start-app,start-export}.ts`, `state/{blockers,export-store}.ts`, `copy/messages.ts`, `routes.tsx`, `ui/components/{ExportButton,PreviewPlayer}.tsx`, `ui/layout/TopBar.tsx`, `ui/styles/*`, `persistence/{entitlement-repo,schema}.ts`, `scripts/check-copy-codes.mjs`, `web/playwright.config.ts`, `tests-e2e/helpers/fake-api.ts`.

**Tests created.** Server: `auth_flow`, `entitlement_rules`, `usage_receipts`, `account_delete`, `account_export`, `rate_limit`, `log_redaction`, `no_media_routes`, `webhook_idempotency`, `billing_events`, `dodo_adapter` (`dodo_adapter.rs` and `tests/fixtures/dodo/*.json` are new rows of TS §5). Web: `auth-machine.test.ts`, `entitlement.test.ts`, `receipt-outbox.test.ts`, and `signin.spec.ts`, `export-free.spec.ts`, `checkout.spec.ts`.

**TS §5 changes,** in the commit that creates each: `http_mailer.rs` → `smtp_mailer.rs`; `dodo_adapter.rs`; `tests/fixtures/dodo/`; `usecases/restore-session.ts` if the tree lacks it; five rows for `docs/v6/`.

**Frozen in V6** (the frozen check, G 0.4): `crates/offcut-{types,plan,scene,render,text,detect,dsp,mp4,wasm-core,wasm-render}/` except DC-11's comments; `fixtures/{golden,labeled,speech}`; `verify/`; `web/src/workers/`; `web/src/models/`; `server/migrations/`. No migration is needed: 0001 holds all six tables. Goldens and snapshots have an empty diff (TS §41.1).

---

## 5. Build order

### 5.1 Prompts

| Prompt | Deliverable | Gate tier | Push |
|---|---|---|---|
| Setup | `v6-build`, documents, DC-12 recorded, draft PR | Counts | Yes |
| P137 | DC-11, known issues 10 and 3 | 1 + one case | No |
| P138 | Dependencies, `Mailer`, `smtp_mailer`, templates, config, `AppState` | 1 | No |
| P139 | `sign.rs`, `issue` (no subscription yet), magic link, verify, access token | 1 | No |
| P140 | Refresh, logout, `AuthUser`, `/me`, CSRF; `auth_flow.rs` complete | 1 | No |
| P141 | `/entitlement`, `/usage/receipts`; three test files | 1 | No |
| P142 | Account delete and export, S-10, retention; `rate_limit`, `log_redaction`, `no_media_routes` | 3 | **First, not waited on** |
| P143 | Billing core with a fake provider: trait, offers, webhook, table; two test files | 1 | No |
| P144 | Real adapter from test-mode payloads, checkout and portal routes | 1 | No |
| P145 | `http.ts`, `api-client`, stores, machine, `effective`, outbox, broadcast, restore; unit tests | 1 | No |
| P146 | `SignInDialog`, `AuthCallbackPage`, routes, copy; `signin.spec.ts` | 2 | No |
| P147 | Blockers, `start-export`, counter, receipt, last-export dialog; `export-free.spec.ts`, DC-12 | 2 | No |
| P148 | `UpgradePrompt`, `AccountPage`, checkout, portal, delete, export; `checkout.spec.ts`; design look | 2 and 3 | **Second, not waited on** |
| P149 | TE-8's 50 mails; TE-9 complete; M1 list walked locally | 1 | No |
| P150 | Full media once, copy-back, merge, deploy, live checks, tag | 4 | **Third, waited on** |

Fourteen numbered prompts and Setup, against BP's rough 10 (TS §41.4's stop-and-ask is at 20). Estimate: 3-5 working days of agent time, plus the founder's steps in the guide. Calendar: merchant approval is not needed for the tag, since V6's gate is a test-mode purchase.

### 5.2 DC-11

Files: `crates/`, `web/src/`, `verify/`, `scripts/`. Only inside comments; a hit in a string or an identifier is reported, not changed. Each hit is also read once, as a sentence, because the map can make a comment wrong about what it means (for instance "V6.6's chapters": chapters are now post-launch; write "post-launch"). Proof:

1. Comments stripped from the old and new trees compare equal (Rust with a small script ignoring `//`, `//!`, `///`; TypeScript likewise; TOML `#`).
2. `pnpm gen:types`, then `sh scripts/check-gen-clean.sh` clean.
3. `cargo test -p offcut-plan -p offcut-scene` unchanged: the catalog and display-list snapshots do not embed comments, and any that fail mean a string was touched.

---

## 6. Mail

### 6.1 Contract

```rust
// mail/mod.rs
#[async_trait]
pub trait Mailer: Send + Sync {
    /// `link` is the whole URL. Neither argument may reach a log.
    async fn send_magic_link(&self, to: &str, link: &str) -> Result<(), MailError>;
}
pub enum MailError { Rejected, Unavailable }   // both become 503 email_unavailable
```

`smtp_mailer.rs` builds one `AsyncSmtpTransport<Tokio1Executor>` with `starttls_relay(host)`, `.port(port)`, `.credentials(...)`, `.timeout(Some(8 s))`, and wraps the send in `tokio::time::timeout`. An error logs `kind` only: its text can hold an address. `templates.rs` builds the message: subject "Your Offcut sign-in link", the 15-minute lifetime stated, one link, "if this was not you, ignore this" in plain text and small HTML.

`AppState` gains `mailer: Arc<dyn Mailer>` and `billing: Arc<dyn BillingProvider>`. `tests/common/mod.rs` gets `FakeMailer`, which stores the last link for a test to read and can be told to fail, and `FakeBilling`.

### 6.2 The port probe (done 2026-11-02)

Read on a throwaway free Render web service, from the public image `docker.io/library/python:3.12-alpine`, holding no secret, deleted afterwards:

```text
2525: open
587: blocked
465: blocked
```

So D-252 stands and `lettre` stays. This reads a TCP connection only: STARTTLS and a login from Render are first proven by the real mail of G 14.6, and a failure there is D-253's stop. To read it again, for instance on another host, set an environment variable `P` to the script below and the Docker command to `python -c exec(__import__(chr(111)+chr(115)).environ[chr(80)])`. The first form, with the script inline in the command, exited on Render with no output: Render does not document how that field parses quotes and spaces, so this one has neither.

```text
import socket,http.server,os;R=[];[(lambda p:(lambda s:(s.settimeout(8),R.append(f'{p}: '+('open' if s.connect_ex(('smtp-relay.brevo.com',p))==0 else 'blocked')),print(R[-1],flush=True)))(socket.socket()))(p) for p in (2525,587,465)];B='\n'.join(R).encode();H=type('H',(http.server.BaseHTTPRequestHandler,),{'do_GET':lambda s:(s.send_response(200),s.end_headers(),s.wfile.write(B))});http.server.HTTPServer(('',int(os.environ.get('PORT','10000'))),H).serve_forever()
```

---

## 7. Sign-in and sessions

### 7.1 `POST /auth/magic-link`

1. `normalize_email`, else 400. 2. `limit(MagicLinkEmail, EmailHash)`. (`MagicLinkIp` is middleware.) 3. Random 32 bytes, base64url; store SHA-256, `expires_at = now + 15 min`. 4. In one statement, mark earlier unused links for that email consumed. 5. `mailer.send_magic_link(normalized, APP_ORIGIN/auth/callback?token=…)`; failure deletes the new row and answers 503. 6. 202, empty. The answer does not depend on whether a user exists: users are made only at verify.

### 7.2 `POST /auth/verify`

One transaction: `SELECT … FOR UPDATE` the row by hash. None or consumed: 400 `link_invalid`. Past `expires_at`: 400 `link_expired`. Otherwise set `consumed_at`; find or create the user (UUIDv7, D-260); create a session with a new refresh token, 60 days. Commit, then `issue()`. Answer `SessionResponse` and `Set-Cookie`.

### 7.3 `POST /auth/refresh` (D-258)

One transaction, `SELECT … FOR UPDATE` on the session whose current or previous hash matches. CSRF first (§7.5).

| The cookie's hash is | Session | Then |
|---|---|---|
| Current, not expired or revoked | Valid | New token T'. `prev_token_hash = current`, `prev_valid_until = now + 30 s`, current = T'. 200 with cookie |
| Previous, `now <= prev_valid_until` | Valid | **Rotates the same way:** new T', `prev_token_hash = current`, `prev_valid_until = now + 30 s`. 200 with cookie |
| Previous, later | Valid | **Reuse: `revoked_at = now`.** 401 `session_expired`, cookie cleared |
| Anything else, no cookie, revoked, expired | | 401 `session_expired`, cookie cleared. Nothing revoked, as nothing identifies a session |

Why the second row rotates: if the first response is lost, the browser still holds the old token, and only a new cookie saves it. In the rare race of two tabs whose responses cross, the browser holds either of the last two tokens, and both are valid for 30 s.

Cookie: `offcut_rt=<token>; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=5184000`. Cleared with `Max-Age=0`. Read by hand from the `Cookie` header: one function, in `session.rs`. `last_used_at` is set on each rotation. `expires_at` does not slide past 60 days from `created_at` **(assumption)**.

### 7.4 Logout, `/me`

`POST /auth/logout`: CSRF; revoke by either hash; clear cookie; 204 always. `GET /me`: `AuthUser`, `limit(Me, User)`, the row, the plan, the summary, a fresh token.

### 7.5 CSRF and errors

The two cookie routes require `X-Offcut-Csrf: 1` and `Origin == APP_ORIGIN`; else 403 `forbidden`. `AppError` gains `Unauthorized`, `Forbidden`, `LinkInvalid`, `LinkExpired`, `SessionExpired`, `EmailUnavailable`, `BillingUnavailable`, each mapping once (D-264). `no-store` is on every API response already.

---

## 8. Entitlement and receipts

`issue()` is TS §22.3, once. `sign_token(claims, key)` in `offcut-entitlement`. `GET /entitlement`: `AuthUser`, limit, a user-row check, `issue`. `POST /usage/receipts`: `AuthUser`, limit, the plan now, `INSERT … ON CONFLICT (export_id) DO NOTHING`, `issue`. Never refused for exceeding a limit. Creator above `CREATOR_FAIR_USE_PER_MONTH` logs D-269's line, with `log::user_tag`.

**A token's month.** `start_of_utc_month`, `end_of_utc_month` from `time`; tests cross a month and a year boundary.

**The public key.** `config/entitlement-public-key.ts`'s production key must verify what the server signs. V5's live test proved it with `PROD_ENTITLEMENT_SIGNING_KEY`; G 8.7 reads it again on the deployed site.

---

## 9. Account deletion and export

As TS §22.7. `USER_TABLES` is D-270. `magic_links` and `platform_waitlist` go by normalized email in the same transaction. The export's `sessions` carry no hash. `purge_once` gains `magic_links` older than 1 day, `webhook_events` older than 90 days, revoked or expired `sessions`, and keeps its one return value, the total. TS §23.3.

---

## 10. Billing and the provider

### 10.1 Contract (REOPENED TS §22.5, D-274, D-275)

```rust
#[async_trait]
pub trait BillingProvider: Send + Sync {
    async fn create_checkout_url(&self, user: UserId, email: &str, offer: Offer, return_url: &str, cancel_url: &str) -> Result<String, BillingError>;
    async fn create_portal_url(&self, customer_id: &str, return_url: &str) -> Result<String, BillingError>;
    async fn cancel_subscription(&self, subscription_id: &str) -> Result<(), BillingError>;
    fn verify_signature(&self, headers: &HeaderMap, raw_body: &[u8], now: UnixSecs) -> Result<(), BillingError>;
    fn parse_event(&self, headers: &HeaderMap, raw_body: &[u8]) -> Result<Option<BillingEvent>, BillingError>;
}
pub enum EventOwner { User(UserId), Customer(String) }
pub struct BillingEvent { pub provider_event_id: String, pub owner: EventOwner, pub customer_id: String,
                          pub subscription_id: Option<String>, pub kind: BillingEventKind,
                          pub interval: Option<BillingInterval>, pub current_period_end: Option<UnixSecs>,
                          pub occurred_at: UnixSecs }
```

Fields a kind does not carry are `None`. Applying a kind that needs a missing one is `billing_event_ignored`, not a failure. Provider error text never reaches a log: `kind` only.

### 10.2 Dodo Payments (the unknowns are read in P144, D-279)

From Dodo's documents, read 2026-11-02:

| Need | Call |
|---|---|
| Checkout | `POST {base}/checkouts`, bearer key, `product_cart: [{product_id, quantity: 1}]`, `customer: {email}`, `return_url`, `cancel_url`, `metadata: {user_id}`. Answer: `checkout_url` |
| Portal | `POST {base}/customers/{id}/customer-portal/session?return_url=…`. Answer: `link` |
| Cancel | `PATCH {base}/subscriptions/{id}` `{"status":"cancelled"}` |
| Webhook | Headers `webhook-id`, `webhook-timestamp`, `webhook-signature`. Body `{business_id, type, timestamp, data}`. Up to 8 attempts over about 27 hours |

| Event | Kind | Read in test mode |
|---|---|---|
| `subscription.active` | `Activated` | `metadata.user_id` present? `next_billing_date` is the period end? Also sent on recovery (D-276)? |
| `subscription.renewed` | `Renewed` | Before `active` on a first payment? (The table ignores it from `none`, and `Activated` still sets everything) |
| `subscription.past_due`, `subscription.on_hold` | `PastDue` | Which of the two comes first |
| `subscription.updated` with `cancel_at_next_billing_date: true` | `CancelScheduled` | Is this the event? Fires on any field, so other changes parse to `None` |
| `subscription.cancelled`, `subscription.expired` | `Canceled` | Is the period end on it? |
| `refund.succeeded` | `Refunded`, `EventOwner::Customer` | Event name and fields. Partial refund: does it end access? Recommendation: only a full one |
| Anything else | `None` | |
| Is `webhook-id` the same across retries? | Dedupe | Delay an endpoint and look. If it changes: stop and ask |

The founder creates, in test mode: three products (monthly 15 USD, annual 144 USD, founding annual 99 USD), an API key and a webhook secret. They go in `.env.deploy` by name; the agent reads them with scripts that print nothing, as in V6 CL Entry 0.1.

How to read payloads before a deploy: make a test purchase and read the delivery log in the merchant's dashboard. Copy each body into `server/tests/fixtures/dodo/<event>.json`, replacing email, name and ids with fixed test values. A test signs them with a test secret, so `dodo_adapter.rs` runs in CI with no network.

### 10.3 Webhook handler (C-19)

Signature, else 401. Parse; `None` is 200. Transaction: insert `webhook_events`; a unique violation is 200. Resolve the owner: `User` through `users`; `Customer` through `subscriptions`; none found is D-272/D-284's 200, with the event row kept. Older than `last_event_at`: ignored. Apply TS §22.5's table with D-276's row. Commit. 200.

`/billing/checkout`: `AuthUser`, limit, offer through `offers.rs`; a founding offer while disabled is 400. `return_url` is `APP_ORIGIN/account?checkout=success`, `cancel_url` `…=cancel`. `/billing/portal`: a subscription row, else 404. Any provider failure: 502 `billing_unavailable`.

---

## 11. The client's foundations

### 11.1 `api-client.ts`: the 12 new functions

`ready()`, `requestMagicLink`, `verifyMagicLink`, `refresh`, `logout`, `getMe`, `getEntitlement`, `postUsageReceipt`, `createCheckout`, `getPortal`, `deleteAccount`, `exportAccount`. The comment "V1 has 3 of the 15" is corrected.

### 11.2 Retry per route

| Route | `retry` | Why |
|---|---|---|
| `ready()` (`/healthz`) | `cold-start`, `onWaking` shows `API_WAKING` | Awaited first by every flow below |
| `magic-link`, `verify` | `none`, after `ready()` | D-259 |
| `refresh` | `cold-start` | Safe by D-258 |
| `me`, `entitlement`, `portal`, `export` | `cold-start` | Reads |
| `usage/receipts` | `none`; the outbox is the retry | 5 s, 30 s, 5 min, then each app start |
| `checkout` | `none`, after `ready()` | Each makes a provider session |
| `delete` | `none`, after `ready()` | The person sees the result |

### 11.3 Auth machine

TS §12.2's table, through `transition()`. `auth-store.ts` holds state, email and the access token in memory. Never in storage. `restore-session.ts`: refresh; 200 signed in and token stored; 401 anonymous; network failure `anonymous_offline`, then a one-shot `online` listener.

### 11.4 Entitlement

`entitlement-store.ts`: token and decoded claims (D-292). `entitlement.ts::effective()`: `freeExportsRemaining = max(0, claims.free_exports_remaining - free rows in the outbox)` (D-267), `plan`, `exp`. It is async-free: `receipt-outbox.ts` mirrors its count into the store. `entitlement-repo.ts` gains `clear()`.

### 11.5 Outbox

`enqueue`, `remove`, `flushOutbox`. Flushes after an export, at app start, on `online`. Timers 5 s, 30 s, 5 min. A 2xx removes the row and sets the token. A 4xx other than 401 also removes it: it can never succeed, and it logs `client_error`. Nothing else removes one (`receipt-outbox.test.ts`, TS §23.3).

### 11.6 Broadcast

`platform/broadcast.ts` wraps one `BroadcastChannel("offcut")`, carrying the single word `signed_in`. No token crosses it.

---

## 12. The gate and the export

`blockers.ts`: `B_NOT_SIGNED_IN` when auth is not `signed_in` and there is no unexpired cached token; `B_ENTITLEMENT_EXPIRED` when there is a token with `exp < now`, reached only while signed in; `B_NO_FREE_EXPORTS` per `effective()`. A signed-out person with only an expired token meets the first. `forCheckout()` is added.

`start-export.ts`, its V5 step 1 and 2 become:

1. `forExport()` with D-290's split. Sign-in, upgrade and last-export answers are UI state in `export-store` (`gate: "sign_in" | "upgrade" | "confirm_last" | null`), which `ExportButton`/`TopBar` render.
2. `B_ENTITLEMENT_EXPIRED`: `getEntitlement()`; success stores it and goes on; failure shows `E_ENTITLEMENT_EXPIRED`.
3. Free with `effective == 1`: the dialog, then on.
4. The token is `entitlement-store`'s. The rest is V5's §13, unchanged.

**The receipt** goes in V5's one place: after a new render delivers, before `saved`, `enqueue(exportId, exportedAs)`; a write failure there logs `client_error` and does not fail the export, since the person has the file. `flushOutbox` after `saved`. A hit writes none. A cancel writes none (C-12).

`export_started`, `export_done` and the verifier's cases do not change. **A cache hit behind `B_NO_FREE_EXPORTS` is blocked:** C-7 runs before C-9. "Download again" still works on a `done` export.

---

## 13. Interface and copy

All copy in `messages.ts` (INV-15), from PS J9 and J12: "Create a free account to export. Free: 3 exports a month, 720p, small watermark. Creator ($15/month): no watermark, 1080p."; "Remove the watermark and export in 1080p: Creator, $15/month."; "Make another clip". Prices through `config/pricing.ts`.

| Component | Rules |
|---|---|
| `ExportButton` | Counter visible before the click on Free ("2 of 3 left"); nothing extra on Creator; nothing when signed out, since the dialog holds the limits |
| `SignInDialog` | Limits above the email field. States: entry, sending (`API_WAKING` after 3 s), sent ("Check your email", resend, spam hint), errors in place. Focus trapped; Escape closes (`link_sent → anonymous`) |
| `AuthCallbackPage` | Strips the token with `history.replaceState` **first**, then verifies. `SIGNED_IN_RETURN_TO_TAB` and a link to `/app`; expired and invalid states offer a new link |
| `UpgradePrompt` | Placements `after_export`, `limit_reached` only (INV-14). Dismissible; once per export. Monthly and annual, no founding row while disabled |
| `AccountPage` | Anonymous: the sign-in form. Signed in: plan, counter, checkout return with polling, manage subscription, export data, delete with a confirm dialog, sign out |
| `TopBar` | One "Account" link. No prompt during processing or editing (PS §11) |

`check-copy-codes.mjs`'s `COPY_PENDING` empties: all 11 codes get copy in the commits that use them; V4's tokens and primitives are used, and the design skill, if available, loads in P146 to P148. One founder look at the captures in P148, as D-235.

`PreviewPlayer.tsx`: `aria-details` only on the Captions tab, and `review-edit`'s case 16 scans all four tabs (known issue 10).

---

## 14. Analytics

| Event | Where |
|---|---|
| `signin_step` `link_requested`, `completed` | `request-magic-link`, `complete-sign-in` |
| `upgrade_prompt` `shown`, `clicked`, `dismissed` | `UpgradePrompt` through a use-case |
| `checkout_step` `started`, `returned_success`, `returned_cancel` | `start-checkout`, `confirm-checkout` |

No event carries an id, an email or a plan beyond the allowlist. Never joined to a user (TS §22.6).

---

## 15. Tests

### 15.1 Server (real Postgres, fake `Mailer` and `BillingProvider`)

| File | Cases |
|---|---|
| `auth_flow.rs` | Link stored only as a hash; second request invalidates the first; expired; used; invalid email; mail failure 503 and no row; verify creates once, then finds; cookie attributes exact; access token verifies, `exp` set; rotation; **previous within 30 s rotates and answers a cookie (D-258)**; previous after 30 s revokes; unknown is 401 with nothing revoked; CSRF header and Origin each refused; logout idempotent; `/me` after delete is 401 |
| `entitlement_rules.rs` | Free 3, 2, 0 floored; only `plan_at_export='free'` counts; month and year boundary; active, past due, canceled within period, past period, refund; claims' 7 fields; token verifies with the public key |
| `usage_receipts.rs` | Insert; repeat changes nothing and has the same shape; another user's export id does nothing to theirs; over the limit accepted; plan recorded; fair-use line once |
| `account_delete.rs` | Rows gone; cancel first; provider failure 502 and nothing deleted; waitlist and links gone; no active subscription means no cancel; idempotent; **`USER_TABLES` complete against `information_schema`** |
| `account_export.rs` | Shape; no hash; receipts in order |
| `rate_limit.rs` | Each group at its limit; 429 with `retry_after_secs`; email key independent of IP |
| `log_redaction.rs` | Marker email, token, cookie and webhook body through the flows: none in captured logs |
| `no_media_routes.rs` | Walks the router: nothing above 64 kB; media types 415 everywhere |
| `webhook_idempotency.rs` | Bad signature 401 and no row; replay 200, no change; unknown type 200; older than `last_event_at` ignored; deleted user 200 |
| `billing_events.rs` | **Every (from, kind) cell**, with D-276's row; each applied cell's fields; ignored cells logged |
| `dodo_adapter.rs` | Fixtures signed with a test secret; Standard Webhooks' published vector; old timestamp refused; each fixture to its kind; unknown product ignored |

### 15.2 Crate

`token_roundtrip.rs`: sign then verify; tamper each segment; the existing TypeScript vector still verifies. `profile.rs`: None → Preview; Creator → 1080p; expired or Free → 720p with watermark.

### 15.3 Client

`auth-machine.test.ts` every legal row and TS §12.3's illegal one. `entitlement.test.ts` D-267 and floor. `receipt-outbox.test.ts` survival, removal rules, timers. `http.test.ts` gains bearer, CSRF and D-287. `blockers.test.ts` gains the three and D-290. `start-export.test.ts` gains receipt on a miss, none on a hit or cancel. `run-pipeline.test.ts` unchanged.

### 15.4 End to end

`fake-api.ts` gains every route, with `/auth/refresh` answering **401 by default**: today's 404 would make every existing media test take a wrong path. Options set session, plan, remaining and failures.

| Spec | Project | Cases |
|---|---|---|
| `signin.spec.ts` | `non-media`, from `/account`: no GPU needed | Limits before the email; two tabs, broadcast signs in the first; expired, invalid; token gone from the URL; waking line after 3 s; email failure shown |
| `checkout.spec.ts` | `non-media` | Navigates to the fake provider URL; return polls to Creator; timeout shows `E_BILLING_PENDING`; cancel return; portal link |
| `export-free.spec.ts` | `media`, shared clip | Counter before the click; 720x1280 with watermark passes all 11; counter falls; receipt once; last-export dialog before the third; fourth shows the prompt and does not render; prompt after a watermarked export, once |
| `export-creator.spec.ts` | `media` | DC-12's one case only |

Running `signin` and `checkout` without a browser GPU also puts them in CI's `pnpm e2e`.

---

## 16. CI and deploy

`ci.yml` gains nothing but `server/tests/fixtures`. `SQLX_OFFLINE` holds: `cargo sqlx prepare --workspace` after each new query, committed in that prompt. **Deploy order (TS §33):** the founder sets Render's new variables first, merges, watches the run. Then registers `https://<render-host>/api/v1/billing/webhook` at the merchant, for test mode.

---

## 17. Experiments

| ID | Who, when | How | Pass | If it fails |
|---|---|---|---|---|
| TE-8 | Agent sends from D1, founder reads, P149 | 50 real template mails with a link to `APP_ORIGIN`: Gmail, Outlook, Yahoo at least. Inbox, spam or missing, in 30 s. The link's host is unchanged | At least 95% in the inbox within 30 s; quota at least 100 a day (Brevo: 300) | D-256 |
| TE-9 | Founder and agent, P144, P149, G 8.7 | Test purchase, renewal, cancel, refund; delayed endpoint | All six kinds observed; a timed-out webhook retried; a test purchase updates the row and the next token says Creator | Second provider behind the trait |

---

## 18. Risks

| Risk | Way out |
|---|---|
| Port 2525 blocked on Render | D-253: HTTP API |
| Gmail sender goes to spam | D-256: domain, founder's call |
| A mail scanner runs the link's script and spends the token | The page POSTs only after load; a scanner that runs script could. Watch `link_invalid` in `client_error`. If real, verify on a click. Not built now |
| `metadata` does not reach webhooks | D-280 |
| Provider declines verification | Second provider behind the trait; the code is unaffected (BP §8.2) |
| Webhook during Render's sleep | Keep-warm, and the provider's retries (§10.2) |
| An existing test takes the new gate | §15.4's defaults; D-267; G item 3 |
| Render swaps instances mid-deploy | No migration in V6 |
| The test-mode merchant on the live site | D-285 |

---

## 19. Hand-over to V6.1

`AuthUser`, `issue()`, `USER_TABLES`, `authed()`, `effective()`, `forExport`'s order and the `Mailer` and `BillingProvider` traits are frozen. V6.1's gate reads `auth-store` for "long videos need an account at the drop". V6.2's routes use `AuthUser` and add two names to `USER_TABLES`.

---

## 20. Exit checklist

- [ ] BP §8.4's server, crate, client and E2E lists pass, plus `dodo_adapter.rs`, `receipt-outbox.test.ts`.
- [ ] Every TS §41.1 proof holds; goldens and snapshots have an empty diff; DC-11 and DC-12 are the only rows used.
- [ ] Counts at or above D-244's.
- [ ] Render's port reading recorded. TE-8 and TE-9 recorded.
- [ ] A real mail signs a real browser in on the deployed site.
- [ ] A test-mode purchase on the deployed site updates `subscriptions`, and the next token says Creator (founder reads production).
- [ ] An export on the deployed site: Free 720p with watermark, then Creator 1080p without, each verified by `verify_mp4.py`.
- [ ] BP §8.3's M1 list walked by the founder.
- [ ] `git grep -nE "\bV6\b" -- web/src crates server` shows no marker of work to do.
- [ ] Copy-back done (§21). Merged, deployed, tagged `v6` by the founder.

---

## 21. Copy-back to the specs (P150, plus TS §5 as files appear)

| Spec | Change |
|---|---|
| TS §5 | `smtp_mailer.rs`; `dodo_adapter.rs`, fixtures; `docs/v6/` rows |
| TS §7, §24.1 | "Email API host" → SMTP relay; merchant API host named |
| TS §10.6 or §23.1 | Outbox row's `plan` (D-267) |
| TS §12.1 | `effective()` subtracts Free rows (D-267) |
| TS §12.5 | Button split (D-290) |
| TS §22.2 | `users.email` normalized (D-260); rotation in the grace (D-258) |
| TS §22.5 | Trait, `EventOwner`, table row (D-274 to D-276) |
| TS §22.8 | Variables (D-251, D-273) |
| TS §39.3 | Token lifetimes, TE-8, TE-9 |
| TS §41.2 | DC-12, once the founder says yes |
| PS §17 | Email passes through the mail provider (the provider named) |
| BP §8, §8A.0 | V6 as built; estimate 14 prompts and the new total |
