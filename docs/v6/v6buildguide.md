# v6buildguide.md - Offcut V6: step-by-step build guide

## What this is

`docs/v6/v6implementation.md` says what each V6 file contains and why. This guide says what to do next, which command to run, and how to know it worked. V6 is built on `v6-build`, with a draft pull request into `main` so that CI runs. It ends with the pull request merged, the real journey working on the deployed site in the merchant's test mode, every box of §20 ticked, and the tag `v6` pushed.

## How to read the references

| Notation | Means |
|---|---|
| `§n` | Section `n` of `docs/v6/v6implementation.md` |
| `TS §n`, `PS §n`, `BP §n` | `technicalspec.md`, `product.md`, `buildplan.md` (v1.3) |
| `V5 §n`, `V5 CL`, `V6 CL` | `docs/v5/v5implementation.md`; the V5 and V6 changelogs |
| `D-n` | A decision: D-241 to D-292 are V6's (§2) |
| `P<nn>` | A prompt of `docs/v6/coding-promptsv6.md` (137 to 150) |
| Tier 0-4 | The test tiers of Step 0.5 |
| D1 | The founder's Windows machine; readings are labelled `d1` |
| "the founder" | The person running the build. Dashboards, production reads, merges and tags are theirs |

## Phase overview

| Phase | Prompt | Covers | Agent time (estimate) | Milestone in one line |
|---|---|---|---|---|
| 0 Setup | Setup | §1A | 30 min | `v6-build`, documents committed, floor counted, draft PR |
| 1 Cleanup | 137 | §5.2 | 1.5 h | Comments renumbered and proven; issues 10, 3 |
| 2 Mail | 138 | §6 | 1.5 h | Port read on Render; `smtp_mailer` sends a real mail |
| 3 Sign-in | 139 | §7.1, §7.2, §8 | 2 h | A link becomes a user, a session and a signed token |
| 4 Sessions | 140 | §7.3-§7.5 | 2 h | Rotation table all tested; `/me` |
| 5 Token and receipts | 141 | §8 | 1 h | `entitlement_rules`, `usage_receipts`, `token_roundtrip` |
| 6 Account | 142 | §9, §15.1 | 1.5 h | Delete, export, S-10; **push 1** |
| 7 Billing core | 143 | §10.1, §10.3 | 2 h | Every cell of the table tested on a fake provider |
| 8 Adapter | 144 | §10.2 | 2 h + founder | Real test-mode payloads; checkout and portal |
| 9 Client core | 145 | §11 | 2.5 h | Machine, stores, outbox, `http.ts` |
| 10 Sign-in UI | 146 | §13 | 2 h | `signin.spec.ts` |
| 11 The gate | 147 | §12 | 2.5 h | `export-free.spec.ts` with all 11 checks |
| 12 Upgrade, account | 148 | §13, §14 | 2.5 h | `checkout.spec.ts`; founder's look; **push 2** |
| 13 Experiments | 149 | §17 | 1 h + founder | TE-8, TE-9 recorded; M1 walked |
| 14 Close | 150 | §20, §21 | 2.5 h + founder | Merged, deployed, live checks, tag |

Total about 26 hours of agent time over 3-5 working days, plus the founder's steps. All estimates.

## Read this first: 12 things that will bite you

1. **Do not renumber one version at a time.** V6.1→V6.2 then V6.2→V6.3 moves every first hit twice. One regex, one pass (D-245).
2. **`http.ts` thinks 502 and 503 mean "waking".** The server's `billing_unavailable` is 502 and `email_unavailable` is 503: without D-287 they retry for 70 s and read as a timeout.
3. **Existing media tests must not meet the new gate.** Three defaults keep them safe: `fake-api` answers `/auth/refresh` with 401 (today 404); a seeded unexpired token passes `B_NOT_SIGNED_IN`; `effective()` subtracts Free rows only (D-267). Check these first if a V5 case fails after P147.
4. **`export-creator.spec.ts`'s "no token" case changes in three lines, and nothing else in that file** (DC-12, D-246): its title, line 127, and one line closing the dialog, because the group is serial. A second failure there is a regression, not a reason for another edit.
5. **A retried verify reads a good link as used** (D-259). Await `ready()`, then one attempt.
6. **Refresh inside the grace must rotate and send a cookie** (D-258, §7.3). The tempting version, "accept it and send nothing", signs people out 30 s after a lost response in a cold start.
7. **`magic_links` has no column for the address as given** (D-260). Do not add a migration.
8. **`deny.toml` refuses `getrandom` until `offcut-api` is a wrapper, and `quoted_printable` until `0BSD` is allowed.** Both go in the commit that adds the dependency.
9. **Reqwest's default TLS brings a second crypto provider.** Use `rustls-no-provider` and install `ring` once, in `main.rs` and in the test app. Two providers panic at the first TLS call.
10. **Render aborts on a missing variable.** The new values are set in its dashboard before the merge (Step 14.4).
11. **A webhook for a deleted user is 200,** not a foreign-key 500 (D-272). Deleting an account always causes one.
12. **Never read `.env.deploy` into a reply, a log or a commit.** Scripts that use it print names and pass or fail only. `git grep` for a key's first characters before each push when one has been handled.

---

## Phase 0 - Setup

### Step 0.1 - Founder, before the agent starts

Done on 2026-11-02: the merchant is Dodo Payments; mail test 1 arrived; DC-12 is yes, "carefully".

### Step 0.2 - The branch

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut
git fetch origin --tags --prune
git rev-parse --short origin/main                 # expect: 4ff3b74 or later
git log --oneline origin/main..v5-build           # expect: one line, 3cae48e
git switch -c v6-build v5-build                   # carries the uncommitted v1.3 specs and docs/v6/
```

### Step 0.3 - The documents

TS §5 gets five `docs/v6/` rows beside the `docs/v5/` ones (line 965 today). `v6changelog.md`'s Entry 1 records the branch, the base, DC-12's answer and the provider's name. Commit `V6 setup: specs v1.3, the V6 documents`; push to `origin v6-build`.

### Step 0.4 - The frozen check

```bash
BASE=$(git rev-parse --short origin/main)
git diff --name-only "$BASE" -- fixtures/golden fixtures/labeled fixtures/speech verify web/src/workers web/src/models server/migrations
# expect: no output, always
git diff --name-only "$BASE" -- '*.snap' '*.png'
# expect: no output, always
```

Crates named frozen in §4 change only in P137, as comments.

### Step 0.5 - The tiers

| Tier | Runs | Used by |
|---|---|---|
| 0 | The new or changed case alone | Every prompt, first |
| 1 | `pnpm check`; `cargo test -p` each crate touched; Vitest | Every prompt |
| 2 | Tier 1, a build, and the named cases: `pnpm e2e:media <spec> --grep "<case>" --max-failures=1` (write `[.]`, never a backslash) | 146, 147, 148 |
| 3 | `pnpm check && pnpm fixtures && pnpm test`, `pnpm e2e` | 142, 148 |
| 4 | Tier 3, full `pnpm e2e:media` on a keyed build, CI waited for | 150 |

Server tests need Docker Desktop running (Postgres on port 9000).

### Step 0.6 - Founder: draft PR

`gh pr create --draft --base main --head v6-build`, or the website. Tell the agent the number.

**Milestone 0.** `git branch --show-current` prints `v6-build`; `gh pr view` shows a draft; Setup's counts match D-244 or the entry says why.

---

## Phase 1 - Cleanup (P137)

1. **Script in the scratchpad, not committed:** one regex over comments only, with the map of D-245.
2. Read every changed line once as a sentence (§5.2); chapters → "post-launch".
3. Prove: comments stripped, old and new compare equal; `pnpm gen:types`; `sh scripts/check-gen-clean.sh`.
4. **Issue 10:** `PreviewPlayer.tsx` sets `aria-details` on the Captions tab only; `review-edit`'s case 16 scans all four tabs, and fails first on the old code.
5. **Issue 3:** read it for 30 minutes at most: does `page.clock.install()` hold the capability check back on the hosted runner? Fix if one line; else leave retries and record what was read.

**Milestone 1.** `git diff --stat "$BASE" -- '*.snap' '*.png'` empty; Tier 1; the one review case passes on a keyed build. Commit `V6 P137: comments through the version map, aria-details, openEditor`.

---

## Phase 2 - Mail (P138)

### Step 2.1 - Founder: the port probe

Done on 2026-11-02 (§6.2): 2525 is open from Render's free service.

### Step 2.2 - The code

Dependencies (§3.1), `deny.toml` (D-250), `config.rs` and its tests, `render.yaml`, `.env` names (placeholder values; the agent changes names only), `mail/*`, `AppState`, `tests/common/mod.rs`'s two fakes (`FakeBilling` panics if called until P143).

### Step 2.3 - One real send through the real code

A temporary `zz_` example or ignored test reads `.env.deploy` by name and sends the template to `MAIL_FROM`, with `https://example.invalid/auth/callback?token=test`. Founder reads the mailbox: inbox or spam, and is the link's host unchanged (D-255). Delete the temporary file.

**Milestone 2.** `cargo deny check` clean; `cargo tree -p offcut-api | grep -E 'native-tls|openssl|aws-lc'` empty; Tier 1. Commit `V6 P138: mail through lettre, config, fakes`.

---

## Phases 3 to 6 - The server (P139 to P142)

Each prompt: write the test file first where the contract is a table, run it alone, run `cargo sqlx prepare --workspace` after new queries, then Tier 1.

| Phase | Done when |
|---|---|
| 3 | `auth_flow.rs`'s link cases green; a token from `issue` verifies with `offcut-entitlement` and `token_roundtrip.rs` passes |
| 4 | `auth_flow.rs` whole: **all four rows of §7.3**, CSRF, cookie attributes exact. Commit `V6 P140` |
| 5 | `entitlement_rules.rs`, `usage_receipts.rs`, crate tests |
| 6 | `account_delete.rs`, `account_export.rs`, `rate_limit.rs`, `log_redaction.rs`, `no_media_routes.rs`. Router comments gone except billing's. Tier 3. **Push 1, not waited on**; P143 reads its result first |

Temporary curl against a local server is allowed once in Phase 4: `cargo run -p offcut-api` with `.env`, a fake-mail build flag is **not** added; use tests instead if that needs one.

---

## Phase 7 - Billing core (P143)

Trait as §10.1. `billing_events.rs` is a table over every (from, kind) cell, written from TS §22.5 plus D-276, before `webhook.rs`. Then `webhook_idempotency.rs`.

**Milestone 7.** Both green on `FakeBilling`. `adapter.rs` does not exist yet. Commit `V6 P143: billing core on a fake provider`.

---

## Phase 8 - The real adapter (P144)

### Step 8.1 - Founder, in the merchant's test mode

1. Create three products: Creator monthly 15 USD, Creator annual 144 USD, founding annual 99 USD.
2. Create an API key and a webhook secret. Put in `.env.deploy`: `BILLING_API_BASE_URL=https://test.dodopayments.com`, `BILLING_API_KEY`, `BILLING_WEBHOOK_SECRET`, `BILLING_PRICE_CREATOR_MONTHLY`, `BILLING_PRICE_CREATOR_ANNUAL`, `BILLING_PRICE_CREATOR_ANNUAL_FOUNDING`.
3. Tell the agent it is done. Paste no value.

### Step 8.2 - One test purchase, created by the adapter

A temporary `zz_` program uses `create_checkout_url` with `metadata.user_id` set to a fixed test UUID, and prints the URL (a test-mode URL, no secret). Founder opens it and pays with the test card, then cancels and refunds in the dashboard.

### Step 8.3 - Read what really came

Founder copies each body from the merchant's webhook delivery log into `server/tests/fixtures/dodo/<event>.json`, or gives the agent the log. If the dashboard shows no bodies, point a test-mode webhook at the §6.2 service, extended to log bodies; the data is test data. The agent scrubs email, name and ids, then answers every row of §10.2's "read in test mode" column in the changelog. **`metadata` missing is D-280's stop; a `webhook-id` that changes on retry is a stop.**

### Step 8.4 - Write it

`adapter.rs`, `offers.rs`'s reverse map, routes, `dodo_adapter.rs`.

**Milestone 8.** `dodo_adapter.rs` green with no network; §10.2's questions answered in the entry. Commit `V6 P144: the provider's adapter from real payloads`.

---

## Phases 9 to 12 - The client (P145 to P148)

| Phase | Steps | Done when |
|---|---|---|
| 9 | `http.ts` (D-286, D-287) with `http.test.ts` first; `api-client.ts`; machine; stores; `effective`; outbox; broadcast; `start-app.ts`'s marker; `fake-api.ts`'s routes with 401 refresh | Vitest up; `landing.spec.ts` still green; one pipeline case still green on a keyed build |
| 10 | Routes; `SignInDialog`; `AuthCallbackPage`; copy; `signin.spec.ts` in `non-media` | Spec green; axe clean on both |
| 11 | `blockers.ts`; `start-export.ts` §12; `ExportButton` counter; last-export dialog; receipt; DC-12; `export-free.spec.ts` | Spec green with 11 checks each export; `export-creator` cases named in Tier 2 pass; `planOf` gone |
| 12 | `UpgradePrompt`, `AccountPage`, use-cases, analytics, `checkout.spec.ts`; `COPY_PENDING` empty; captures in `~/v6-review/`, founder looks | Tier 3; **push 2, not waited on** |

---

## Phase 13 - Experiments (P149)

1. **TE-8.** Founder lists at least 3 mailboxes across Gmail, Outlook and Yahoo. Agent sends 50 real template mails from D1 across them, with a harmless link to `APP_ORIGIN`, paced 2 s apart. Founder counts inbox, spam and missing, and checks the time on a sample and the link's host. Below 95% inbox: **stop and ask** (D-256).
2. **TE-9.** Renewal, cancel, refund and a delayed endpoint, as far as test mode allows before a deploy; the rest in Step 14.6.
3. **M1 walk.** BP §8.3 on D1 against local `offcut-api`, real mail, test-mode merchant. Founder ticks.

**Milestone 13.** `docs/v6/experiments.md` holds both. Commit `V6 P149: TE-8, TE-9, the M1 list`.

---

## Phase 14 - Close (P150)

| Step | Who | What |
|---|---|---|
| 14.1 | Agent | `git grep -nE "\bV6\b" -- web/src crates server`: no marker of work to do. Copy-back §21. BP §8A.0's estimate |
| 14.2 | Agent | Tier 4, once. Docker closed for no bench: V6 changes no render path, so **no bench** |
| 14.3 | Agent | Push 3; `gh pr checks <pr> --watch`, in the background |
| 14.4 | **Founder** | Render: add `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `MAIL_FROM`, `BILLING_API_BASE_URL`, real `BILLING_*`. Keep `MAIL_API_KEY` until the deploy is healthy, then remove it |
| 14.5 | **Founder** | `gh pr ready`, merge, watch the `main` run |
| 14.6 | **Founder** | Register the webhook URL for test mode. On the live site: real mail signs in; Free export; test purchase; Creator export; refund; delete a test account. Agent runs `verify_mp4.py` on files the founder saves outside the repository, if asked. Founder reads `subscriptions` in production |
| 14.7 | Agent | `healthz` shows the merge; header check; closing entry |
| 14.8 | **Founder** | `git tag -a v6 -m "V6: accounts, limits, payments" <merge> && git push origin v6` |

**Milestone 14.** §20 all ticked or named. V6.1 opens its own branch.

---

## Quick reference

```bash
# Every terminal: Git Bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
export BENCH_DEVICE=d1
pnpm check                      # all static checks
cargo test -p offcut-api --test auth_flow     # one server file (Docker running)
cargo sqlx prepare --workspace  # after any new query
pnpm build:e2e                  # keyed build for media suites
pnpm e2e                        # non-media, including signin and checkout
pnpm e2e:media <spec> --grep "<case>" --max-failures=1
```

**Assumptions and gaps.** Render's Docker-command field is assumed to take the probe's command as written. Dodo's unknowns are listed in §10.2.
