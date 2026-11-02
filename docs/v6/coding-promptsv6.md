# Offcut V6 — coding-promptsv6.md (Prompts 137–150)

Source documents: `docs/v6/v6implementation.md` (written **§n**; its decisions are **D-n**, D-241 to D-305) and `docs/v6/v6buildguide.md` (written **G n.m**, **G item n** for an item of its "Read this first"). `TS`, `PS`, `BP` are the v1.3 specs. These prompts were written after reading the code at `3cae48e` on 2026-11-02.

**How to use.** Paste the **Standard Agent Block (SAB)**, then one prompt, into your coding agent. Run them in order: Setup, then 137 to 150. Do not start a prompt until the one before it has its commit and every acceptance box ticked. Commits are titled `V6 P<nn>: <title>`. Steps marked **Human** are the founder's. Each prompt's **Effort** line is the reasoning effort to set: High where money, sessions or a contract are at stake, Medium for records and specified screens.

**Design skill.** In P146, P147 and P148, if the agent has a frontend or design skill, it loads it before writing interface code. V4's tokens, primitives and copy win over any default it suggests.

---

# Standard Agent Block (SAB) — prepend to every prompt

**Every terminal (Git Bash, never PowerShell or WSL)**

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
export BENCH_DEVICE=d1
BASE=4ff3b74
git branch --show-current     # v6-build, always (Setup creates it)
```

Server tests need Postgres on port 9000, a Windows service on D1, and not Docker (D-305). `gh auth status` shows a login.

**Before modifying anything**

1. Read the plan sections and guide steps this prompt cites, every file it touches, and the previous prompt's entry in `docs/v6/v6changelog.md`.
2. Search before creating (`git grep -n "<name>"`). `offcut-api-types` already holds every DTO, `ApiErrorCode` and `RouteGroup`; `offcut-types` the limits. `normalize_email`, `user_tag`, `by_ip`, the token's `encode` and `signing_input` exist.
3. Verify that the previous prompts' outputs behave as this prompt assumes. If not, make the smallest compatible correction and report it.
4. **Precedence.** A decision D-241 to D-305 is binding, also where it refines TS; D-1 to D-240 stand unless a V6 decision replaces them. Otherwise TS wins over the plan and PS over both. If sources disagree in a way these do not settle, or a signature is missing, **stop and list it**.
5. **TS §41 binds.** Any protected proof that would change without a row of §41.2, any new runtime, browser host or paid service, any failed experiment offering a choice: **stop and ask, with a recommendation.**

**During implementation**

1. Implement only this prompt's scope. A new file under `crates/`, `server/`, `web/`, `scripts/`, `verify/` or `.github/` that TS §5 does not list fails `check-file-tree.mjs`; add the row in the same commit only for the files §4 names.
2. **Frozen:** G 0.4's check stays empty. Goldens and snapshots do not change. No migration. The crates of §4 change only as comments in P137.
3. **Reopened contracts of code or a protected proof are exactly:** D-286, D-287; DC-12's one case. Say **REOPENED CONTRACT** in the entry. Decisions that change spec text are listed in §21 and copied back in P150.
4. No source file over 700 lines above its test module. Rust non-test code: no `unwrap`, `expect`, `panic!`, slice indexing. TypeScript: no `any`, `!`, `@ts-ignore`, empty `catch`, or inline style. No disabled lint, no weakened or deleted test. Dependencies only as §3.1 lists.
5. User-facing text only in `copy/messages.ts`. `fetch` only in the allowed files. A temporary file is `zz-…` or `zz_…` and is deleted before the gate.
6. **Secrets.** Never print, log, commit or paste a value from `.env.deploy`, a token, a cookie or a link. A script using `.env.deploy` prints names and pass or fail only, and lives in the scratchpad. **Never query the production database.** A server log line names a `kind`, never provider or SMTP error text: it can hold an address.
7. **Privacy.** Never stage `testclips/**`, `fixtures/.gen/**`, `corpus/**` data, `web/test-results/**` or `.env*`. Fixtures in `server/tests/fixtures/dodo/` hold no real email, name or id.
8. Long runs start detached and are read from a log tail. Append to the changelog with the Edit tool, never a heredoc holding backticks.
9. **Branch.** Commit on `v6-build` only. Push only where the prompt says. Never push to `main`, merge, tag or mark the pull request ready.

**When existing code disagrees with the plan.** If it blocks the prompt, fix the minimum and record why. If not, report it under "Found, not fixed".

**The gate.** G 0.5's tier, the new case alone first; then:

```bash
git branch --show-current
git status --porcelain | grep -c "zz[-_]"        # 0
env | grep -c VITE_ENTITLEMENT_TEST_PUBLIC_KEY   # 0 before the commit
git diff --name-only "$BASE" -- fixtures/golden fixtures/labeled fixtures/speech verify ':!verify/README.md' web/src/workers web/src/models server/migrations '*.snap'   # no output
git ls-files --cached | grep -E '(^|/)\.env'     # no output
```

Counts never fall below the previous entry's; the floor is D-244's. Add a short entry to `v6changelog.md` in the same commit: what changed, what differs from the plan, **REOPENED CONTRACT** lines, counts, what is open, "Found, not fixed". Rewrite its two top tables when an item opens or closes. Commit with the trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Reply with files changed, commands and results, deviations, **Human** steps open, issues not fixed.

---

# Who does what

| When | Human step |
|---|---|
| Before Setup | Done on 2026-11-02 (G 0.1) |
| P138 | Read one real mail (G 2.3); one real send from Render (G 2.4); mailboxes for TE-8, and counting them (G 2.5) |
| P144 | Test-mode products and key (G 8.1); the logging endpoint and its secret (G 8.2); the founder's rows of the event table (G 8.3); delete the service (G 8.4) |
| P148 | One look at `~/v6-review/` |
| P149 | The M1 walk's local rows (G 13) |
| P150 | Render variables; ready and merge; webhook registration; live checks, saved exports and the production read; M1's checkout row; the tag (G 14) |

---

# Prompts

## Setup prompt (V6 00) — The branch, the documents, the floor

**Effort: Medium.**

**Objective.** `v6-build` exists with the v1.3 specs and V6's documents as its first commit, and CI can run.

**Before.** On `v5-build` at `3cae48e`, with `docs/technicalspec.md`, `docs/product.md`, `docs/buildplan.md` modified and `docs/v6/` untracked. G 0.1's answers are in.

**Implement.** G 0.2 to G 0.4. TS §5's `docs/v6/` rows, TS §41.2's DC-12. Record the answers in Entry 1.

**Acceptance.**
- [ ] `git log --oneline origin/main..HEAD` shows `3cae48e` and the new commit.
- [ ] `node scripts/check-file-tree.mjs` passes. Rust and Vitest counts read, at D-244's floor.
- [ ] Pushed. Draft pull request 10 is in the entry.

## Prompt 137 (V6 01/14) — Comments through the version map; known issues 10 and 3

**Effort: Medium.**

**Objective.** DC-11 exactly, and V5's two carried issues.

**Implement.** G Phase 1, §5.2, D-245, with `verify/README.md` line 16 by name. Issue 10 test-first. Issue 3 for at most 30 minutes.

**Do not.** Touch a token outside a comment. Rewrite closed changelogs or plans.

**Acceptance.**
- [ ] Comments-stripped trees compare equal; say how, and the count of lines changed.
- [ ] `sh scripts/check-gen-clean.sh` clean after `pnpm gen:types`.
- [ ] `review-edit`'s case 16 fails on the old `PreviewPlayer.tsx`, passes on the new, on all four tabs.
- [ ] No `.snap` or `.png` in the diff. V6.1-style numbers in code now read as TS §42's.

## Prompt 138 (V6 02/14) — Mail through lettre, config, fakes

**Effort: High.**

**Before.** G 2.1 is done: 2525 is open from Render (§6.2).

**Implement.** §3, §6, D-248 to D-256, D-298, D-304, G 2.2 to G 2.5. `Mailer`, `MailError`, `smtp_mailer.rs`, `templates.rs`. `billing/provider.rs` with `UnconfiguredBilling`, `billing/mod.rs`. `Config` per §3.2, with `Debug` redaction and its tests. `AppState` per §6.1, built in `main.rs`, with `rustls`'s `ring` provider installed once. `FakeMailer`, `FakeBilling` in `tests/common/mod.rs`. `render.yaml`. `.env` names, values untouched. TS §5's `smtp_mailer.rs` row.

**Do not.** Add a route. Log an address or link.

**Acceptance.**
- [ ] `cargo deny check` clean; `cargo tree -p offcut-api` holds no `native-tls`, `openssl` or `aws-lc`.
- [ ] `Config` tests cover each new variable missing and `SMTP_PORT` malformed, naming the variable and never the value.
- [ ] **Human:** one real mail read: inbox or spam; link host unchanged.
- [ ] **Human:** `sent` read from Render, and the service deleted (G 2.4). A failure is D-253's stop.
- [ ] TE-8 recorded, with the visible From. Below 95% inbox or a changed link host is a stop (D-255, D-256).

## Prompt 139 (V6 03/14) — A link becomes a user, a session and a token

**Effort: High.**

**Implement.** §7.1, §7.2, §8's signing, D-257 to D-261, D-264, D-265, D-266. `sign.rs`, `token_roundtrip.rs`, `profile.rs` (crate). `db/{users,magic_links,sessions}.rs`. `auth/{mod,magic_link,session,access_token}.rs` to the extent these two routes need. `entitlement/issue.rs` with its subscription read, through a small `db/subscriptions.rs` read and `db/usage_receipts.rs` count. `AppError`'s new variants. `limit()` in `rate_limit.rs`. Routes `/auth/magic-link`, `/auth/verify`. `cargo sqlx prepare`.

**Do not.** Refresh, logout, billing.

**Acceptance.**
- [ ] `auth_flow.rs`'s link cases of §15.1 pass; the stored row holds only a hash.
- [ ] `verify` is transactional: two racing verifies, one success.
- [ ] `cargo tree -p offcut-wasm-render` shows `offcut-entitlement` without `sign`.
- [ ] A token from `issue` verifies with the public key of `ENTITLEMENT_SIGNING_KEY`.

## Prompt 140 (V6 04/14) — Rotation, logout, the extractor, `/me`

**Effort: High.** This is the prompt most able to sign paying users out.

**Implement.** §7.3 to §7.5. `extract.rs`'s `AuthUser`, the rest of `session.rs`, `POST /auth/refresh`, `/auth/logout`, `GET /me`. Write §7.3's table as tests first.

**Acceptance.**
- [ ] All four rows of §7.3 are tests, each asserting the database row **and** the cookie; the second row answers a new cookie and leaves the previous hash and its time as they were; the fourth sends no `Set-Cookie`.
- [ ] The next refresh with that new cookie succeeds.
- [ ] **Three** concurrent refreshes with one cookie: all 200, no revocation, and the last one's token refreshes.
- [ ] A token rotated away, used after 30 s (move the row's time in the test, do not sleep): 401, no `Set-Cookie`, nothing revoked; the current token still works.
- [ ] CSRF: header missing, `Origin` wrong, each 403. Cookie string asserted exactly.
- [ ] `/me` for a deleted user: 401.

## Prompt 141 (V6 05/14) — The token and receipts

**Effort: Medium.**

**Implement.** §8, D-268's server half, D-269. `GET /entitlement`, `POST /usage/receipts`, `usage/mod.rs`, `entitlement/mod.rs`. TS §22.3 and §22.4 exactly.

**Acceptance.**
- [ ] `entitlement_rules.rs` and `usage_receipts.rs` as §15.1, with month and year boundaries.
- [ ] A receipt over the limit is accepted and returns a token.

## Prompt 142 (V6 06/14) — Account, S-10, retention; the guard tests

**Effort: Medium.**

**Implement.** §9, D-270 to D-272. `account/{delete,export}.rs`, `USER_TABLES`, `purge_once`. `rate_limit.rs`, `log_redaction.rs`, `no_media_routes.rs`. `FakeBilling` has `cancel_subscription`'s success and failure.

**Acceptance.**
- [ ] `account_delete.rs` fails when a table with `user_id` is left out of `USER_TABLES` (try it, then undo); `account_export.rs` shows a row of every listed table.
- [ ] `log_redaction.rs` runs the auth, receipt and analytics flows with markers; none in the output.
- [ ] Tier 3 green. **Push 1, not waited on.**

## Prompt 143 (V6 07/14) — Billing core on a fake provider

**Effort: High.**

**Before.** Read push 1's CI first; fix anything red.

**Implement.** §10.1, §10.3, D-274 to D-277, D-283, D-284, D-299 to D-301. `provider.rs`'s real events, `offers.rs`, `webhook.rs`, the rest of `db/subscriptions.rs`, `db/webhook_events.rs`, `POST /billing/webhook`. `billing_events.rs` first, every cell.

**Do not.** `adapter.rs`, or any call to a real provider.

**Acceptance.**
- [ ] `billing_events.rs` covers every (from, kind) cell, D-276's and D-301's included; ignored cells change nothing; another `subscription_id` is ignored except on `Activated`.
- [ ] `webhook_idempotency.rs` as §15.1; the delete-then-`Canceled` case answers 200.
- [ ] The dedupe insert and the apply are one transaction: a forced failure in the apply leaves no event row.

## Prompt 144 (V6 08/14) — The provider's adapter, from real payloads

**Effort: High.**

**Before.** **Human:** G 8.1, G 8.2.

**Implement.** G 8.3 to G 8.5, §10.2, D-273, D-278 to D-282. Answer every "read in test mode" row in the entry **before** writing the mapping. `adapter.rs`; `POST /billing/checkout`; `GET /billing/portal`; `dodo_adapter.rs`, with Standard Webhooks' published vector. `FakeBilling` stays for route tests.

**Stop and ask if:** `metadata.user_id` does not reach subscription events (D-280); `webhook-id` changes across retries; a scheduled cancel sends no readable event; the refund cannot be tied to a customer.

**Acceptance.**
- [ ] Every fixture parses to its kind; fixtures hold no real personal data.
- [ ] A body changed by one byte, an old timestamp and a wrong secret each fail.
- [ ] Founding offer refused while disabled. Checkout refused for `active` and `past_due` (D-300). Provider failure is 502 `billing_unavailable`.
- [ ] Every test subscription cancelled; the logging service deleted (**Human**).
- [ ] `cargo tree -p offcut-api` still has one crypto provider.

## Prompt 145 (V6 09/14) — The client's foundations

**Effort: High.**

**Implement.** §11, D-267, D-268, D-286 to D-289, D-291, D-292, D-295 to D-297. `http.ts` with tests first. `api-client.ts`, `auth-store`, `auth-machine`, `entitlement-store`, `entitlement.ts`, `receipt-outbox.ts`, `broadcast.ts`, `pricing.ts`, `restore-session.ts`, `start-app.ts`'s marker, `entitlement-repo.clear`, `schema.ts`'s outbox value. `fake-api.ts`: every route; `/auth/refresh` **401 by default**.

**Do not.** Any interface. Any change to `start-export.ts`.

**Acceptance.**
- [ ] `auth-machine.test.ts`, `entitlement.test.ts`, `receipt-outbox.test.ts` pass; `http.test.ts` covers bearer, CSRF, D-287.
- [ ] A 502 `billing_unavailable` makes one attempt and gives `E_BILLING_UNAVAILABLE`, in a test.
- [ ] Two callers meeting 401 cause one refresh, in a test. Refresh takes the lock, and stops retrying at 25 s, in a test.
- [ ] `auth-machine.test.ts` walks the callback tab's path: `unknown`, verify, `signed_in`, with no throw.
- [ ] `pnpm e2e` green; one `pipeline-preview` case green on a keyed build, showing nothing waits on `restoreSession`.

## Prompt 146 (V6 10/14) — Sign-in on screen

**Effort: Medium.**

**Implement.** §13's dialog and callback, `routes.tsx`, `/account` as a minimal page holding the sign-in form, `request-magic-link`, `complete-sign-in`, `sign-out`, copy for `E_AUTH_*`, `API_WAKING`, `signin_step`. `signin.spec.ts` in `non-media` (update `web/playwright.config.ts`'s `testMatch`).

**Acceptance.**
- [ ] §15.4's cases; token removed from the URL before the verify call; no refresh sent from the callback; axe at 0 on dialog and callback.
- [ ] Copy-coverage passes with `E_AUTH_*` off `COPY_PENDING`.

## Prompt 147 (V6 11/14) — The gate, the counter, the receipt

**Effort: High.**

**Implement.** §12, D-246, D-290, D-292 to D-294, D-302. `blockers.ts`; `start-export.ts`; `export-store`'s `gate`; `ExportButton`; the last-export dialog; the receipt in V5's one place; DC-12's one case; `blockers.test.ts`, `start-export.test.ts`, with its one named case of D-246; `export-free.spec.ts`. Remove `planOf`, `setUnavailable`, `messages.export.unavailable`.

**Do not.** `UpgradePrompt`'s final form (a stub placement is enough until P148). Any other assertion of `export-creator.spec.ts`.

**Acceptance.**
- [ ] `export-free.spec.ts` as §15.4; every export passes `verify_mp4.py`'s 11 checks.
- [ ] Named `export-creator` and `timeline-preserved` cases pass unchanged, except DC-12's.
- [ ] Lines 142 and 321 of `export-creator.spec.ts` pass as they are (D-293). The button's accessible name is still exactly "Export" with the counter showing.
- [ ] **DC-12, carefully.** `git diff "$BASE" -- web/tests-e2e/export-creator.spec.ts` touches only that one case: its title, line 127's assertion, one line closing the dialog before line 134. The entry quotes the diff. Lines 128-135 are untouched and pass. The next case, "a token signed with another key", passes after it. Anything more is a stop.
- [ ] A receipt on a miss; none on a hit or a cancel.
- [ ] The button is enabled when signed out, and the click opens the dialog.

## Prompt 148 (V6 12/14) — Upgrade, checkout, the account page

**Effort: Medium.**

**Implement.** §13's remainder, §14, D-300, D-302. `UpgradePrompt`, `AccountPage`, `start-checkout`, `confirm-checkout`, `open-billing-portal`, `delete-account`, `export-account-data`, `TopBar`'s link, the last copy, `checkout.spec.ts`. Captures of each new state at two widths in `~/v6-review/`.

**Acceptance.**
- [ ] `COPY_PENDING` empty; its check and every analytics allowlist test pass.
- [ ] `checkout.spec.ts` as §15.4. `UpgradePrompt`'s placement type has two members; it is non-modal, outside `export-outcome`, and takes no focus. `export-creator`'s case "summary and the question" passes unchanged in Tier 2.
- [ ] Axe at 0; every control works by keyboard.
- [ ] Tier 3 green. **Push 2, not waited on.** **Human:** one look.

## Prompt 149 (V6 13/14) — TE-9, the M1 list's local rows

**Effort: Medium.**

**Before.** Read push 2's CI; fix anything red.

**Implement.** G Phase 13. `docs/v6/experiments.md`. No product code unless a reading demands it.

**Acceptance.**
- [ ] TE-9's rows as far as test mode allows.
- [ ] **Human:** BP §8.3's rows other than checkout ticked on D1, with G 13's key set-up.

## Prompt 150 (V6 14/14) — Full run, copy-back, merge, deploy, tag

**Effort: Medium.**

**Implement.** G Phase 14, §20, §21. The one full media run, on a keyed build, detached. No bench: no render path changed. Then wait on CI.

**Human, in order:** G 14.4, 14.5, 14.6, 14.8. The agent does not run them.

**Acceptance.**
- [ ] Full media and CI green; counts at or above D-244's, in the entry.
- [ ] §21 copied back. No `V6` marker of work to do.
- [ ] After the merge: `healthz` shows it; header check; the live journey and the production read reported by the founder; TE-9's rest.
- [ ] Closing entry written, with every box ticked or named. **Human:** tag `v6`.

---

# Forbidden shortcuts

| Shortcut | Why not |
|---|---|
| Accept a previous refresh token and send no cookie, or move the previous hash on each use | Signs people out after a lost response or a third request (D-258) |
| Retry refresh past 25 s | A late retry reads as reuse and revokes the session (D-297) |
| Run `restoreSession` on the callback tab | Its answer can overwrite the new session (D-296) |
| Put the counter inside the button, or make `UpgradePrompt` modal | Breaks protected cases (D-302) |
| Retry `/auth/verify` | Spends a good link (D-259) |
| Guess the provider's payloads | Loses paid users silently (D-279) |
| A 500 on a webhook for a missing user | Eight retries for nothing (D-272) |
| Read `.env.deploy` into context | Secrets rule |
| Add a migration for a convenience | §4; instances swap uncoordinated |
| Change another `export-creator` assertion to pass | TS §41: stop and ask |
| A second home for a plan or limit computation | `issue()`, `effective()`, `blockers.ts` are the only ones |
| A prompt during processing or editing | PS §11, INV-14 |
