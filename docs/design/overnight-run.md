# Overnight run: the Offcut design release, from Setup to live

You are a fresh Claude Code session, and you work alone overnight. The founder of Offcut is asleep. When they wake up, they want **the new landing page live on https://offcut-one.vercel.app**, with **nothing that works today broken**. Every decision the plan leaves to the founder is already made, or is delegated to you below. Nobody can answer a question tonight. Where the plan says "stop and ask", follow §6 of this prompt instead: take the safe path, record it, and carry on with what is safe.

Work carefully rather than fast. A night that ends with a smaller release, live and unbroken, is a success. A night that ends with everything shipped and something broken is a failure.

---

## 1. Where everything is

- **Repository:** `C:\Users\Mayan\Desktop\sh2clips\offcut` (Git Bash: `/c/Users/Mayan/Desktop/sh2clips/offcut`). GitHub: `mayanpathak/Offcut`. `gh` is logged in.
- **Branch:** `design-build`, starting at `8d94a06`, which is `origin/main` plus V6's closing entry. The plan commits are `e50f753` and `bd62b9e`. In the frozen check, `BASE=8d94a06`.
- **Live site:** https://offcut-one.vercel.app. **API:** https://offcut-api.onrender.com.
- **Asset host (R2):** the one host in `web/vercel.json`'s security policy.
- **The release's documents, in `docs/design/`.** Read all five before any work:
  1. `designimplementation.md`: the plan. Decisions D-316 to D-335; §2 has the no-break rules; §12 the tests; §14 the captions; §18 the founder's answers.
  2. `designbuildguide.md`: phases, commands, tiers, the frozen check, and "16 things that will bite you".
  3. `coding-promptsdesign.md`: the Standard Agent Block (SAB), Setup, and Prompts 151 to 160. **Follow each prompt as written, with the SAB's rules**, except where this file overrides it (§3, §5, §6, §7).
  4. `designchangelog.md`: one entry per prompt, in the same commit.
  5. `remotion-prompts.md`: how the clips were made.
- **The clips:** in `C:\Users\Mayan\Desktop\sh2clips\offcut-remotion\out\`, seven web files with their masters and `.webp` posters. They were already checked (DCL Entry 2). All pass except the pixel format (`yuvj420p`), which P156 re-encodes. **Read only:** never change that folder.
- **The specs:** `docs/technicalspec.md` (TS), `docs/product.md` (PS), `docs/buildplan.md` (BP), and `docs/v6/v6changelog.md` (V6 CL).
- **Memory.** Your memory folder for this project loads automatically. Read `MEMORY.md`'s entries on the shell quirks, the env files, local Postgres, the test-time budget, the media build and the version-branch rule.

---

## 2. What the release is

All of it is in `designimplementation.md`. In short:

- **Part A (P151-P156).**
  - A new landing page on the editor's dark system, with Inter and JetBrains Mono for the timecodes.
  - The concept is "the page is a short": a captioned headline, timecode eyebrows, a scroll progress bar and a playhead.
  - It is animated with Motion (React side) and GSAP (timelines, loaded with `import()`).
  - It names every feature that works today (§4) and marks every post-V6 feature "Coming soon" (§5).
  - Seven Remotion clips are served from the asset host.
  - "Start free" links lead into `/app`.
  - Nothing says "not open yet", which is what Dodo's reviewers objected to.
- **Part C (P157).** Settings, legal, account, callback and unsupported move onto the dark tokens. This is CSS and one attribute, with no logic.
- **Review (P158).** axe, the captures, and design review 3, done by you (§5).
- **Part B (P159).** Bottom captions sit lower (option A, "bottom-hug", DC-13). This regenerates goldens and snapshots and needs one full media run.
- **Close (P160).** Copy-back, merge, the deploy checks, and the tag.

---

## 3. What you are authorised to do tonight

The founder gave these on 2026-11-05 ("take the design decisions yourself, everything is fine from my end, but don't break logic"). They hold for this release only.

**You may:**

1. **Take every design decision** the plan leaves open: layout details, spacing, motion timing, colours within the tokens, and wording within the drafts' meaning. The founder's answers to F-1 to F-9 are in §18 and stand as written: the H1 "Say it once. Post it finished.", Creator's line "Creator goes on sale soon.", caption option A, Part C yes, the JetBrains Mono accent, the og image, GSAP's licence (after the read), and all seven clips.
2. **Upload the seven re-encoded clips** to the asset host's `media/` folder with `scripts/upload-assets.sh`. **The founder's go for P156 Step 6.3 is given by this prompt.** Upload only the seven re-encoded `.mp4` files, once. Never delete or overwrite anything on the asset host.
3. **Push `design-build`** and any new branch named in §7.
4. **Open pull requests, mark them ready, and merge them into `main` with `gh pr merge <n> --merge`.** Do this only when every gate of §7.2 is green. A merge to `main` deploys: CI runs `ci`, then `e2e-media`, then `deploy-api`, then `deploy-web`, with a smoke test against the deployment.
5. **Re-run a failed CI job once** if it is the known flake: `e2e-media` failing on the model download or `openEditor`, known issue 3. Use `gh run rerun <id> --failed`.
6. **Roll back** with a revert pull request, if production is broken after a merge (§8).
7. **Push the tag `design`** on the merge commit, after the deploy checks pass.

**You may NOT, under any circumstances:**

- push to `main` directly;
- force-push;
- rewrite published history;
- delete branches or tags;
- use Vercel's Redeploy button, Vercel's dashboard, or Render's deploy hook by hand;
- query the production database;
- touch Dodo Payments (the resubmission is the founder's);
- change `web/vercel.json`, `server/**`, `verify/**`, the workers, the pipeline, or any file the frozen check of the guide's Step 0.2 lists, except under the exceptions a prompt names;
- print, log, commit or paste any value from `.env`, `.env.deploy`, a token, a cookie or a link;
- add a dependency other than `gsap` and `motion`;
- add an analytics event;
- edit an existing test case, except the one pricing case §12.4 declares;
- delete anything on the asset host;
- start any service that costs money.

---

## 4. The rules you check after every prompt

These come from the SAB, `designimplementation.md` §2 and the guide's "bites". The ones that matter most:

1. **No logic changes.**
   - `routes.tsx`, `main.tsx`, `App.tsx`, every use-case, store, worker, persistence, net and server file, `DropZone.tsx`, `NotifyMeForm.tsx` and the whole editor stay byte-for-byte the same.
   - `LandingPage.tsx`'s navigation effect stays character for character, and `capabilityLine()` moves verbatim.
   - The guide's Step 0.2 frozen check must print nothing, apart from each prompt's named exceptions.
2. **Before and after.**
   - Run the prompt's tier before its first edit and keep the result. Run it again at the end: every case that passed before still passes, and its code is unchanged.
   - The only existing case that may change is the pricing case, exactly as §12.4 declares, in P153.
   - A failing existing case means **the code is wrong**: fix the code.
3. **Unique strings (§12.1).** These must each match exactly one element on `/`:
   - the hero heading;
   - `supportedBrowsers`;
   - `privacyLine` ("Your video stays on your computer.");
   - the "What leaves your device" link;
   - "Try a sample";
   - "Notify me";
   - the email label;
   - the pricing heading and its sentence;
   - each footer link.

   Playwright matches by substring, so never repeat them. There is one `<form>` on `/`.
4. **Content is never hidden** by an initial animation state (rule 2.5): content may move, and only decoration may fade. Reduced motion gives a complete, static page. `page.clock` freezes animation frames, and two existing cases use it.
5. **The security policy** has no inline styles. Never put `style=` in JSX, call `setAttribute("style")`, add `<style>` elements, or use Motion's `layout` or `popLayout`. Write through refs and `style.setProperty`, which go through the CSS object model and are allowed.
6. **Copy.** Every word lives in `copy/` (`react/jsx-no-literals`). No digit is typed into a sentence: numbers come from `LIMITS` and `PRICES_USD`.
7. **Files.** Every new file needs its TS §5 row in the same commit (`check-file-tree.mjs`). No file goes over 700 lines.
8. **Tests only grow.** Setup records the baseline, and every entry records its counts. The app shell must stay at or under 500 kB gzip.
9. **Privacy of files.** Never stage `testclips/**`, `fixtures/.gen/**`, `corpus/**`, `web/test-results/**`, `.env*`, or anything in `~/design-review/` or `~/design-upload/`.
10. **One commit per prompt**, titled `Design P<nnn>: <title>`. Each commit holds its DCL entry and ends with the trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

---

## 5. The founder's steps, and what you do in their place

| Step | The plan says | Tonight |
|---|---|---|
| P156 Step 6.3: the upload's go | Ask the founder | **Given.** Upload the seven re-encoded files once. First run `grep -c '^ASSET_S3_' .env.deploy`, which prints a count only. If the names are missing, or the upload fails, **do not upload anything else**: ship with the posters and CSS illustrations (D-325, `LANDING_CLIPS` left empty or partial) and record open item "clips not uploaded" |
| P158: design review 3 | The founder reads the captures and the copy | **You do it** with the rubric below. Read every capture with the Read tool (it shows images), fix what fails, capture again, and record the result as "self-review, rubric passed" in place of the founder's "go" |
| P160: the merge | The founder | **You do it**, under §7 |
| P160: the Dodo resubmission | The founder | **Not tonight.** Leave it as open item 4 for the morning |
| P160: the tag | The founder | **You do it** after the deploy checks pass |

**The self-review rubric (P158).** Read the captures at 1280 × 800, 1920 × 1080 and 390 × 844, under normal and reduced motion. Every answer must be yes:

1. Does it answer the creator's complaint ("extremely basic, not appealing, should be animated, premium, should say what Offcut offers")?
   - At 1280 × 800, the first screen shows the headline, the subhead, "Start free" and the hero clip in its phone.
   - Nothing looks like default browser styling.
2. Is every feature of §4 marked for the page visible somewhere, in plain words? Does every row of §5 carry "Coming soon"?
3. Are text, CTAs and cards free of overlap, of cut-off and truncated words, and of sideways scrolling at 390 px?
4. Are spacing and alignment consistent: one content width, even section rhythm, and card grids that line up?
5. Is contrast readable everywhere (axe has no serious or critical findings)? Is the yellow pill used only in the H1, the final call and the Captions card?
6. Is there no placeholder text, no invented number or testimonial, and no other company's name or logo?
7. Under reduced motion, is everything present and still?
8. Do the drop zone, the "Try a sample" button and the email form still look like what they are, and still work (their existing cases pass)?

If an item fails, fix it in P158 and capture again. Two rounds at most, then record what is left as an open item.

---

## 6. When the plan says "stop and ask"

Nobody can answer tonight. Take the safe path and record each case under "Open items" in DCL.

| Situation | What you do |
|---|---|
| An existing test fails and you cannot make it pass without editing it | Revert that prompt's change to the last green commit (`git revert`, or `git restore` before the commit). Drop the feature that caused it, and record it. Never edit the test |
| GSAP's licence says more than "do not build a Webflow competitor" | Do not use GSAP. Remove the dependency, build M1-M10 with Motion and CSS where they stay simple, drop the pin (M9) and the playhead (M3), and record it |
| A security policy violation that you cannot remove | Drop the animation that causes it |
| The app shell goes over 500 kB | Cut motion features (M3, M9 first) until it is under. Do not raise the budget again |
| Part C breaks `signin` or `checkout`, or axe on those pages | Revert P157's commit and ship without Part C |
| Part B (P159): the stills show option A reading worse, a line on the watermark, or anything outside the safe area; or the full media run fails because of the caption change | Do not ship Part B. Leave its branch unmerged (§7.4) and record it. The landing page does not depend on it |
| A flaky CI job | Re-run it once (`gh run rerun <id> --failed`). If it fails again, read its log (`gh run view <id> --log-failed`), fix it if it is ours, otherwise record it, and do not merge |
| Anything about secrets, production data, Dodo, Render or Vercel settings | Do nothing. Record it for the morning |
| You are unsure whether something is logic | Treat it as logic: do not change it |

---

## 7. The order of the night, and the two merges

The landing page goes live first. The captions go second, on their own pull request, because they need the long full media run and touch protected goldens. Either can go live without the other.

### 7.1 Part A, Part C and the review on `design-build`

1. Run **Setup**: the baseline in DCL Entry 3 and the draft pull request.
   - The `cargo test --workspace` server tests need local Postgres on port 9000 (a Windows service on D1, not Docker).
   - Run long commands detached and read them from a log tail.
2. Run **P151, P152, P153, P154, P155, P156, P157 and P158** in order. Each one:
   - reads first;
   - runs its tier before and after;
   - passes the frozen check;
   - gets its own commit and DCL entry.

   **One push for the batch** goes after P158 (memory: test locally first, one CI run per batch). If you want CI's verdict earlier, push once after P156 too, and no more often than that.
3. **P160's copy-back for Parts A and C** goes into the same branch as a last commit, "Design P160a: copy-back (Parts A and C)". It covers TS §5, §13, §39.3 item 24 (500 kB and the reading), TDR-26, INV-15, PS §10 J1 and the BP line. DC-13 is not part of it.

### 7.2 The gates before the first merge (all must be true)

Check every one, and write the list with its evidence into DCL before you merge:

- [ ] Locally: `pnpm check`; `cargo test --workspace` (unchanged counts, since Part A touches no Rust); Vitest; `pnpm build` (`check-hosts.mjs` ok, shell at or under 500 kB); `pnpm e2e`. Every case passes, and counts are at or above Setup's baseline.
- [ ] L-1 to L-12 pass, as far as they were built (L-12 only with Part C).
- [ ] The 17 unchanged landing cases, `signin.spec.ts` and `checkout.spec.ts` pass with no change to their code. Show `git diff 8d94a06 -- web/tests-e2e/signin.spec.ts web/tests-e2e/checkout.spec.ts` (empty) and the `landing.spec.ts` diff (only the declared pricing lines plus new cases).
- [ ] The targeted media cases of P152 and P157 pass on a keyed build (`pnpm build:e2e`).
- [ ] The frozen check prints nothing, apart from Part C's named files.
- [ ] `git diff --stat 8d94a06...design-build` lists only §10.1's files, Part C's files and docs.
- [ ] The CI run on the pull request's head is green, all jobs including `e2e-media`, which runs on pull requests (known flake: one re-run allowed).
- [ ] The self-review rubric (§5) passed.

Then:

```bash
gh pr ready <n>
gh pr merge <n> --merge
```

Watch the `main` run in the background (`gh run watch <id>`, or poll `gh run list --branch main -L 3` every 10 minutes). It must finish `ci`, `e2e-media`, `deploy-api` and `deploy-web`, including the job's own smoke test against the deployment.

- If `deploy-web` times out waiting for the API version, re-run it once after `curl -s https://offcut-api.onrender.com/api/v1/healthz` answers. That happened before, at V6.
- `e2e-media` may flake once; re-run it.

### 7.3 After the first deploy

- `E2E_BASE_URL=https://offcut-one.vercel.app pnpm --filter web exec playwright test --project=non-media --grep @smoke`: green.
- `curl -sI https://offcut-one.vercel.app/`: the security policy is exactly as in `web/vercel.json`, with isolation headers present.
- Run a script that loads `/` in Chrome with Playwright against the live site:
  - the H1 reads "Say it once. Post it finished.";
  - every `LoopVideo` reaches `readyState >= 2`;
  - no console error and no security-policy violation;
  - "Start free" leads to `/app`, which shows the drop zone or the unsupported page.
  - Save screenshots to `~/design-review/live/` (never staged).
- Check the editor is unharmed: on the live `/app`, the drop zone shows and "Try a sample" is enabled. Do not export and do not sign in.
- If any of these fail because of this release, roll back (§8).

If all pass, **tag:**

```bash
git tag -a design -m "Design release: landing page, clips" <merge sha>
git push origin design
```

### 7.4 Part B, the lower captions, on their own branch

1. `git fetch origin && git switch -c design-captions origin/main`. This starts from the merge, which already holds Parts A and C.
2. Run **P159** exactly as written:
   - the stills first, looked at with the Read tool;
   - option A in `captions.rs` (plus the `placement.rs` helper);
   - the unit tests;
   - snapshots and goldens regenerated, every diff read;
   - the DC-13 row in TS §41.2;
   - then **Tier 4**: `cargo test --workspace`, Vitest, `pnpm build`, `pnpm e2e`, then `pnpm build:e2e` and the **full** `pnpm e2e:media`, detached, read from its log. Expect a long run; memory has the test-time budget.
3. If every step passes:
   - push the branch;
   - open the pull request "Design P159: lower Bottom captions (DC-13)";
   - wait for its CI, all green (one flake re-run allowed);
   - check that the gates of §7.2 that apply hold (the frozen check with P159's exceptions; `export-free` and `export-creator` pass all 11 `verify_mp4.py` checks);
   - merge, watch the `main` run and the deploy, and run §7.3's live checks again;
   - move nothing else, and do not re-tag. Add DCL's line "DC-13 live at <sha>".
4. If any step fails and the fix is not plainly inside the caption code:
   - leave the branch pushed and its pull request **draft**;
   - record why in DCL;
   - do not merge.

   The landing page is already live. This is the expected safe outcome if the night runs short.

**Time.** Part A, Part C and the first merge come first. Start Part B only after the first deploy passes §7.3. If the full media run cannot finish before 07:00 local time, leave Part B's pull request as a draft with its state written down.

---

## 8. Rollback (only if production is broken by this release)

Broken means at least one of these is true:

- `@smoke` fails against the live site;
- `/` or `/app` shows a blank page or a script error;
- the security policy differs from `vercel.json`;
- the editor's drop zone is missing on `/app`.

Then:

1. `git switch -c revert-design origin/main && git revert -m 1 <merge sha> --no-edit`.
2. Push it, open a pull request "Revert the design release", and wait for CI.
3. Merge it.
4. Watch the deploy, then run §7.3's checks again. They must show the old page working.
5. Do not tag. Record everything in DCL and in the morning report.

Never try to fix production forward at night with a quick new commit on `main`.

---

## 9. Environment notes (this machine is Windows, D1)

- **Shells.** Use Git Bash for repository work. The SAB's first block runs in every new terminal:

  ```bash
  cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a
  export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
  ```

- **Quoting.** Bash breaks on backticks inside double quotes. Write commit messages and pull request bodies to a file in the scratchpad and pass them with `-F` or `--body-file`. In PowerShell, `git commit -F -` with a here-string does not work: use a file.
- **Line endings.** Write files with LF line endings. Python on Windows writes CRLF unless it opens files with `newline=''`.
- **Changelog.** Append to it with the Edit tool, not with a heredoc that holds backticks.
- **Tools.** `ffmpeg` and `ffprobe` are on PATH (WinGet links). `pyftsubset` comes with `pip install fonttools brotli`, into the user's Python 3.13, if missing.
- **Postgres.** Local Postgres runs on port 9000, as a Windows service. Docker is not needed tonight.
- **Builds.** Media suites need `pnpm build:e2e` (the test mode, with the test key from `.env`). The non-media suites and `check-hosts.mjs` use `pnpm build`. Never deploy a `build:e2e` output, and never commit `dist`.
- **Long runs.** Run them detached, write their logs to the scratchpad, and read the tail. Do not hold a foreground command for more than 10 minutes.
- **Timers.** Use `gh run watch` or a polling loop with `sleep 600` for CI, in a background task.
- **gitleaks.** It runs in CI and can flag a text like `lavapipe:`. If it flags something that is not a secret, fix the text rather than adding to the ignore file, unless the V4 pattern applies.

---

## 10. Reporting

- **Per prompt.** Write a DCL entry in the same commit: what changed, what differs from the plan, DECLARED CHANGE lines, counts, the shell size, open items, and "Found, not fixed".
- **At the end.** Write a final DCL entry, "Morning report, 2026-11-05". It is also your last message, and holds:
  1. **Live or not:** the merge commit(s), the tag, the URL, and the result of §7.3's checks, with the paths of the live screenshots.
  2. **What shipped and what didn't:** Parts A, B and C, and the seven clips each, with the reason for anything held back.
  3. **Every decision you took** under §3.1 or §6, one line each.
  4. **Counts:** Setup's baseline against the final counts (Rust, Vitest, `pnpm e2e`, media), and the app-shell size before and after.
  5. **For the founder:**
     - the Dodo resubmission (V6 CL open item 10);
     - open item 17 (hide the in-app offers until live mode, its own change);
     - a re-render of the clips with real footage if wanted;
     - anything in Open items.
  6. **Found, not fixed.**

Begin by reading the five documents of §1, `MEMORY.md`'s relevant entries and `docs/v6/v6changelog.md`'s open items. Then run Setup.
