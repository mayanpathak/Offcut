# Overnight run: Offcut V6.1 "Long source", from Setup to merged, deployed dark and tagged

You are a fresh Claude Code session, and you work alone overnight. The founder of Offcut is asleep. When they wake up, they want **V6.1 implemented, merged into `main`, deployed and tagged `v6.1`**, with **nothing that works today broken**. V6.1 ships **dark**: its two flags, `long_video` and `high_res`, stay off in production, so the live site behaves exactly as it does now, and the founder turns `high_res` on in the morning (§8). Every decision the plan leaves to the founder is already made (plan §15: "yes to all", 2026-11-08), or is delegated to you below. Nobody can answer a question tonight. Where the plan or a prompt says "stop and ask", follow §6: take the safe path, record it, and carry on with what is safe.

Work carefully rather than fast. **The order of what matters tonight:** (1) nothing that works breaks, in production or in the tests; (2) whatever is merged is whole: no half-built release reaches `main`; (3) V6.1 is merged and tagged. A night that ends with the pull request green, unmerged and fully reported is an acceptable outcome. A night that ends with `main` broken is a failure.

---

## 0. Before the founder sleeps (the founder's checklist)

The founder ticks these, then pastes this file into a new session with permissions set so that the session does not wait on prompts (auto mode).

- [ ] The laptop (D1) is on power; Windows sleep is off ("Never" for screen and sleep on power); the lid stays open or "Do nothing" on close.
- [ ] Local Postgres is running (the Windows service, port 9000).
- [ ] `gh auth status` shows the login; Chrome is installed.
- [ ] `C:\Users\Mayan\Desktop\sh2clips\offcut` is on `cloud-build` with the uncommitted v1.4 documents and `docs/v6.1/` (as left on 2026-11-08).
- [ ] Optional (F-2): the phone HDR clip is copied to `C:\Users\Mayan\Desktop\sh2clips\testclips_private\hdr\` (outside the repository). If it is not there, TE-21 is read on the generated file only.
- [ ] Nothing else is running that uses the GPU heavily (the media suites need it).

---

## 1. Where everything is

- **Repository:** `C:\Users\Mayan\Desktop\sh2clips\offcut` (Git Bash: `/c/Users/Mayan/Desktop/sh2clips/offcut`). GitHub: `mayanpathak/Offcut`. `gh` is logged in.
- **Base:** `origin/main` at `2568a6e` (the cloud release's merge). The working tree is on `cloud-build` (`afb82bf`) with the uncommitted documents. Setup makes `v6.1-build` from `origin/main`. In the frozen check, `BASE=2568a6e`.
- **Live site:** https://offcut-one.vercel.app. **API:** https://offcut-api.onrender.com.
- **The release's documents, in `docs/v6.1/`.** Read all four before any work:
  1. `v6.1implementation.md`: the plan. Decisions D-383 to D-432; §2 the no-break rules; §11 the tests; §15 the founder's answers; §17 the exit checklist.
  2. `v6.1buildguide.md`: phases, commands, tiers, the frozen and additive checks, and "22 things that will bite you".
  3. `coding-promptsv6.1.md`: the Standard Agent Block (SAB), Setup, and Prompts 171 to 182. **Follow each prompt as written, with the SAB's rules**, except where this file overrides it (§3, §5, §6, §7).
  4. This file.
- **The changelog:** `docs/v6/v6changelog.md` (V6 CL), one entry per prompt, in the same commit. Its Entries 21-24 are the v1.4 decisions, the revision, the plan and the founder's answers.
- **The specs:** `docs/technicalspec.md` (TS, read §41 and §43 first), `docs/product.md` (PS), `docs/buildplan.md` (BP §8A.0, §8A.1), all v1.4.
- **The model for an unattended night:** `docs/design/overnight-run.md` and DCL Entry 15 (its morning report). Read them for tone and practice; this file's rules win.
- **Memory.** Your memory folder loads automatically. Read `MEMORY.md`'s entries on the shell quirks, env files, local Postgres, test-time budget, media build, target folder size, version-branch rule and cloud transcription.

---

## 2. What the release is

All of it is in the plan. In short:

- **Two pages.** `/app` stays the clip page exactly as today, plus (flag on) one link to `/app/long` and a "Make shorts from it" button on the "too long" panel. `/app/long` takes videos over 90 s and up to 45 minutes.
- **Reading in place.** The dropped file is never copied: the demuxer reads it through `FileReaderSync` (a new WASM entry), its sound is decoded in windows, resampled in a stream to 16 kHz and written to OPFS, resumably.
- **Cloud only.** The sound goes to `POST /transcribe` in 90 s windows (5 s overlap, two in flight), each checkpointed; a failure pauses with a reason and Continue; the windows merge into a master transcript, shown at the end.
- **The clip page with `high_res`.** 4K and decodable HEVC accepted; 16 GB file limit.
- **Server.** A window header with its own limits; the audio budget from configuration; three analytics events; `no-store` on the API through Vercel.
- **Flags off by default.** Production's build has both off; a second e2e build (`dist-flags/`) tests them on.

---

## 3. What you are authorised to do tonight

The founder gave these on 2026-11-08 ("yes to all … when I wake up tomorrow the feature must be implemented and merged, strictly without breaking, and do everything carefully"). They hold for this release only.

**You may:**

1. **Take every implementation decision** the plan leaves open, inside its decisions. Write each as an "As read" note under the decision it refines, and list it in the morning report.
2. **Make real Groq calls** in P171 and P181 with the looped reference clip's audio only (F-1), about 5,400 s in all, through the scratch proxy (G Phase 11). Never more than 3,000 s in one hour.
3. **Push `v6.1-build`.**
4. **Open the pull request, mark it ready, and merge it into `main` with `gh pr merge <n> --merge`**, only when every gate of §7.2 is green. A merge to `main` deploys: CI runs `ci`, then `e2e-media`, then `deploy-api`, then `deploy-web`, with a smoke test against the deployment.
5. **Re-run a failed CI job once** if it is a known flake (CT CL known issues 1 and 2: `openEditor` on the hosted runner; lavapipe's SIGSEGV after the goldens): `gh run rerun <id> --failed`.
6. **Roll back** with a revert pull request, if production is broken after the merge (§9).
7. **Push the tag `v6.1`** on the merge commit, after the live checks pass.

**You may NOT, under any circumstances:**

- push to `main` directly, force-push, rewrite published history, or delete branches or tags;
- **turn any flag on in production**: never set `OFFCUT_FEATURES_ON` anywhere but `build:e2e:flags`; never touch Vercel's or Render's settings or dashboards, Render's deploy hook, or Vercel's Redeploy button;
- set Render variables (V6.1 needs none: its two server variables are optional and stay unset);
- query the production database; touch Dodo Payments; change Groq's console;
- source `.env.deploy` whole, or print, log, commit or paste any value from `.env`, `.env.deploy`, a token, a cookie or a link;
- send the founder's HDR clip, `testclips/v3testclip*`, `testclips/WIN_*` or any corpus clip anywhere; stage anything under `testclips*`, `fixtures/.gen*`, `corpus/`, `web/test-results/`, `web/dist*`;
- edit an existing test case, except the one `rejections.spec.ts` assertion DC-1 declares (P180);
- change a golden, a snapshot, a frozen file of G 0.4, or a function D-387 names;
- add a dependency, a server migration, or a service that costs money;
- widen what the product accepts beyond the plan: `hdr_ok` stays `false` whatever TE-21 reads.

---

## 4. The rules you check after every prompt

These come from the SAB, the plan's §2 and the guide's "bites". The ones that matter most:

1. **The clip path is untouchable.** G 0.4's frozen check prints nothing; its additive check prints only the lines it names. `run-pipeline.ts`, `transcribe-cloud.ts`, `import.ts`, `audio-decode.ts`, `whisper-runtime.ts`, `word-timestamps.ts`, `validate_probe`, `resample_mono`, `open_demuxer`, `probe_and_validate` are unchanged.
2. **Before and after.** Run the prompt's tier before its first edit and keep the result; run it again at the end. Every case that passed before still passes, and its code is unchanged. A failing existing case means **the code is wrong**: fix the code, never the test.
3. **Flags.** The production build (`pnpm build`) has `long_video` and `high_res` `false`. Check the bundle after any change to `features.ts`, `vite.config.ts` or `package.json`.
4. **Storage.** `DB_VERSION` stays 1; only `sources-repo.ts` opens structure version 2; a test proves the clip path leaves the database at version 1.
5. **Privacy.** A `File`'s name is never read; a `File` is never stored; no test or log holds a transcript word beyond Script A's published text.
6. **Copy.** Every word in `copy/long.ts` as §10.5 approved it. `messages.ts` at or under 700 lines.
7. **Files.** Every new file has its TS §5 row in the same commit; no file over 700 lines.
8. **Tests only grow.** Setup records the baseline; every entry records its counts.
9. **One commit per prompt**, titled `V6.1 P<nnn>: <title>`, holding its V6 CL entry, with the trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
10. **`target/`.** Per-crate tests per prompt; the workspace only in Setup and P182; `cargo clean -p` if it passes 30 GB.

---

## 5. The founder's steps, and what you do in their place

| Step | The plan says | Tonight |
|---|---|---|
| F-1 to F-5 | Answered | "Yes to all" (plan §15). F-5's copy is used exactly as §10.5 drafts it |
| P179, G 9.4: the look | The founder reads the captures | **You do it** with the rubric below; record "self-review, rubric passed" |
| P181: the local walk | The founder walks the local build | **You do it** as a scripted walk on `dist-flags/` (P181 step 5) |
| P181: TE-20's sleep | A real sleep | **Simulated** (G 11.3); the real one is a morning step |
| P182: ready, merge, tag | The founder | **You do it**, under §7 |
| F-3: `high_res` on in production | After V6.1's checks | **Not tonight.** It is the founder's morning step (§8); you state whether TE-17's 4K read allows it |

**The self-review rubric (P179).** Read every capture of the long page (empty, gate signed out, gate checking, gate unavailable, progress in each stage, paused for each reason, awaiting the file, a different file refused, the transcript) at 1280 × 800 and 1920 × 1080, with the Read tool. Every answer must be yes:

1. Does each state say, in its own words from §10.5, what is happening and what to do next?
2. Does it look like the editor (same tokens, type, spacing, cards) and nothing like default browser styling?
3. Is nothing overlapping, cut off or scrolling sideways at 1280 px?
4. Is contrast readable (axe: no serious or critical findings)?
5. Is there no placeholder text, no invented number, no file name anywhere?
6. Are Pause, Cancel and Continue where a person looks for them, and is only one primary button shown at a time?
7. On `/app` with the flag on: is the link quiet (a text link under the drop zone, not a second call to action)?

Two rounds of fixes at most; what is left becomes an open item.

---

## 6. When the plan says "stop and ask"

Nobody can answer tonight. Take the safe path, record each case under "Open items" in V6 CL, and say it in the morning report.

| Situation | What you do |
|---|---|
| An existing test fails and you cannot make it pass without editing it | Revert that prompt's change to its last green state. If the failing part is essential to V6.1, **do not merge tonight**: finish what is safe, push, leave the pull request draft, report |
| G 0.4's frozen or additive check shows a change you cannot remove | The same: revert to green; if essential, do not merge |
| `StreamResampler` cannot equal `resample_mono` exactly | G 3.1's fallback (the long path only), recorded. The clip path never uses it |
| D1's Chrome decodes no HEVC (P171) | `high_res` means H.264 up to 4K only; HEVC stays `REJECT_HEVC`; recorded |
| Window seams cut words (P171, TE-18) | Overlap 10 s (`CLOUD_OVERLAP_S`), `MAX_SOURCE_WINDOWS` recomputed, recorded |
| Peak memory over 1.5 GB on the 45-minute file (P181) | Smaller decode batches and read cache, once; if still over, record it as a stop item for the founder. **The merge can still go ahead**: the long path is dark in production. F-3 waits |
| TE-20 shows a lost checkpoint | Fix it if the cause is in V6.1's code; otherwise record it. It does not block a dark merge if `long-resume.spec.ts` passes |
| TE-21 reads HDR as fine | Record it; `hdr_ok` stays `false` |
| The fixtures add over 90 s to `pnpm fixtures` | Smaller fixtures (shorter long file: 6 minutes still exercises five windows); recorded |
| A flaky CI job | Re-run it once. If it fails again, read its log (`gh run view <id> --log-failed`), fix it if it is ours, otherwise record it, and **do not merge** |
| CI's second build or the `media-flags` job will not run on the hosted runner | Keep the flags-on suites as a D1-only gate tonight (they ran in P180 and P182), record it, and merge only if `dist/`'s CI is fully green |
| Anything about secrets, production data, Dodo, Groq's console, Render or Vercel settings | Do nothing. Record it for the morning |
| You are unsure whether something changes the clip path | Treat it as if it does: do not change it |
| A prompt runs past twice its estimate (TS §41.4) | Finish the smallest safe slice that keeps every earlier gate green; record it; continue with the next prompt only if what is left is still whole. If not, stop building, push, leave the pull request draft, and report |

---

## 7. The order of the night, and the merge

### 7.1 Build on `v6.1-build`

1. **Setup** (baseline in V6 CL Entry 25, draft pull request).
2. **P171 to P181** in order. Each one: think first (the prompts file's rule), read, tier before and after, frozen and additive checks, its own commit and V6 CL entry. Pushes after P176 and P179 (not waited on; read their CI result before P182).
3. **P182**: copy-back, Tier 4, push 3, CI waited on.

### 7.2 The gates before the merge (all must be true)

Write the list with its evidence into V6 CL before you merge:

- [ ] Locally: `pnpm check`; `cargo test --workspace`; Vitest; `pnpm build` (`check-hosts.mjs` ok; the bundle's `long_video` and `high_res` `false`); `pnpm e2e`. Every case passes; counts at or above Setup's baseline.
- [ ] **The full `pnpm e2e:media` on `dist/` passed once** (the protected path, as production runs it); media count at or above 112 passed with the one existing skip.
- [ ] **`FLAG_SUITES` on `dist-flags/` passed** (`route-long`, `high-res`, `long-resume`, `flags-identity`, `rejections`).
- [ ] G 0.4's frozen check prints nothing; the additive check prints only its named lines.
- [ ] `git diff --name-only $BASE...v6.1-build -- web/tests-e2e/` lists only new files, `fake-api.ts`, `long-windows.ts`, `playwright.config.ts` and `rejections.spec.ts`; `git diff $BASE -- web/tests-e2e/rejections.spec.ts` is the one declared assertion; `git diff $BASE -- server/tests/` adds cases only.
- [ ] `git diff --stat $BASE...v6.1-build` lists only the plan's §4 files, docs and the fixtures script.
- [ ] Goldens and snapshots: `git diff --name-only $BASE -- '*.snap' '*.png'` empty.
- [ ] The CI run on the pull request's head is green, all jobs (one re-run allowed for known issues 1-2).
- [ ] The self-review rubric passed (or its open items recorded).
- [ ] `git grep -nE "gsk_[A-Za-z0-9]{20}"` empty; nothing private staged.
- [ ] **Time:** the merge starts before 06:30 local time, so the `main` run (about 35 minutes) and the live checks finish before the founder wakes. Otherwise leave the pull request ready but unmerged and report.

Then:

```bash
gh pr ready <n>
gh pr merge <n> --merge
```

Watch the `main` run in the background (`gh run watch <id>`, or `gh run list --branch main -L 3` every 10 minutes). It must finish `ci`, `e2e-media`, `deploy-api` and `deploy-web` with its smoke. If `deploy-web` times out waiting for the API's version, re-run it once after `curl -s https://offcut-api.onrender.com/api/v1/healthz` answers (it happened at V6).

### 7.3 After the deploy (the plan's §12.2)

- `curl -s https://offcut-one.vercel.app/api/v1/healthz`: `ok`, the merge's version, `cloud_transcribe: true`.
- `curl -sI https://offcut-one.vercel.app/api/v1/me`: 401 and `Cache-Control: no-store` (D-429).
- An unauthenticated `POST /api/v1/transcribe` with `X-Offcut-Window: 0/2` and a tiny `audio/ogg` body: 401 from our API.
- `E2E_BASE_URL=https://offcut-one.vercel.app pnpm --filter web exec playwright test --project=non-media --grep @smoke`: green.
- A Playwright script against the live site, screenshots to `~/v6.1-review/live/` (never staged): `/` loads with no console error; "Start free" leads to `/app`, which shows the drop zone (or the unsupported page) and **no long-video link**; `/app/long` redirects to `/app`; dropping nothing, signing in to nothing, exporting nothing.
- If any of these fail because of this release: roll back (§9).

If all pass, **tag:**

```bash
git tag -a v6.1 -m "V6.1: long source (deployed dark)" <merge sha>
git push origin v6.1
```

Then the closing entry and morning report (§10) on `v6.1-build`, pushed.

---

## 8. What the founder does in the morning (write it into the report)

1. Read the morning report (V6 CL's closing entry; also your last message).
2. **F-3:** if the report says TE-17's 4K clip read passed, turn `high_res` on: Vercel → the project → Settings → Environment Variables → `OFFCUT_FEATURES_ON` = `high_res` (Production), then re-run the `main` run's `deploy-web` job (`gh run rerun <id> --job <deploy-web job id>`), which pulls the variable, builds and runs the test-key guard. Never the Redeploy button. Check `/app` takes a 4K clip.
3. **TE-20's real sleep:** with `pnpm build:e2e:flags` and `vite preview` of `dist-flags/`, start a long video, close the lid for five minutes, open it, and see the job pause and continue.
4. Any open item the report lists.
5. `long_video` stays off in production until V6.4's alpha (F-4).

---

## 9. Rollback (only if production is broken by this release)

Broken means at least one of these is true: `@smoke` fails against the live site; `/` or `/app` shows a blank page or a script error; `/api/v1/healthz` fails or reports the old version after the deploy finished; the editor's drop zone is missing on `/app`; a clip-path request the site used to make now fails.

Then:

1. `git switch -c revert-v6.1 origin/main && git revert -m 1 <merge sha> --no-edit`.
2. Push it, open a pull request "Revert V6.1", wait for CI.
3. Merge it.
4. Watch the deploy, then run §7.3's checks again. They must show today's site working.
5. Do not tag. Record everything in V6 CL and in the morning report.

**The database trap does not apply tonight:** production's flags are off, so no user's browser database was upgraded to structure version 2 (plan §12.4). Never try to fix production forward at night with a quick commit on `main`.

---

## 10. Environment notes (this machine is Windows, D1)

- **Shells.** Git Bash for repository work; the SAB's first block in every new terminal.
- **Quoting.** Bash breaks on backticks inside double quotes. Write commit messages and pull request bodies to a file in the scratchpad and pass them with `-F` or `--body-file`.
- **Line endings.** LF. Python on Windows writes CRLF unless it opens files with `newline=''`; edit docs with the Edit tool.
- **Tools.** `ffmpeg` and `ffprobe` on PATH (WinGet links). `verify_mp4.py` needs Python 3.13 first on `PATH` (the SAB sets it).
- **Postgres** on port 9000, a Windows service. Docker is not needed.
- **Builds.** Media suites need the keyed builds (`pnpm build:e2e`, `pnpm build:e2e:flags`, the key exported first, unset before committing). Non-media suites and `check-hosts.mjs` use `pnpm build`. Never deploy a keyed build; never commit `dist` or `dist-flags`.
- **Long runs.** Detached, logs to the scratchpad, read the tail. No foreground command over 10 minutes. The full media run is about 40 minutes on D1.
- **CI.** `gh run watch` or a polling loop with `sleep 600`, in a background task.
- **gitleaks** runs in CI and can flag text like `lavapipe:`; fix the text rather than the ignore file.
- **The Groq proxy** reads `TRANSCRIBE_API_KEY` alone (G quick reference); it is a scratch file, never committed.

---

## 11. Reporting

- **Per prompt.** A V6 CL entry in the same commit: what changed, what differs from the plan, REOPENED CONTRACT and DECLARED CHANGE lines, every real Groq call, counts, open items, "Found, not fixed".
- **At the end.** V6 CL's closing entry, "V6.1 closed: morning report, 2026-11-09". It is also your last message, and holds:
  1. **Merged or not:** the merge commit, the `main` run's id, the tag, the result of §7.3's checks with the screenshots' paths. If not merged: why, and exactly what is left.
  2. **What was built,** prompt by prompt, one line each, and anything held back.
  3. **Every decision you took** under §3.1 or §6, one line each, with the plan line it refines.
  4. **Counts:** Setup's baseline against the final (Rust, Vitest, `pnpm e2e`, media on `dist/`, `FLAG_SUITES` on `dist-flags/`).
  5. **Experiments:** TE-17 (and F-3's verdict in one sentence), TE-18, TE-20 (simulated), TE-21; every real Groq call's total seconds.
  6. **For the founder:** §8's steps, in order; open items.
  7. **Found, not fixed.**

Begin by reading the four documents of §1, `MEMORY.md`'s relevant entries and V6 CL's top tables and Entries 21-24. Then run Setup.

---

## 12. The founder's answers before sleeping (2026-11-08, 04:50 IST)

The session started at 04:46, so §7.2's "before 06:30" could not hold. The founder answered these before sleeping. Where they differ from the sections above, **this section wins**.

| # | Question | Answer |
|---|---|---|
| A-1 | When may the agent merge? | **Whenever every §7.2 gate is green, any time on 2026-11-08 or 2026-11-09, but never after 22:00 IST**, so a deploy is never left unwatched at midnight. This replaces §7.2's 06:30 line. The work goes on after the founder wakes; if 22:00 passes, the pull request stays ready and unmerged until the next morning |
| A-2 | The HDR clip (F-2) | Not supplied. TE-21 is read on `rej_hdr_hlg.mov` only; `hdr_ok` stays `false` |
| A-3 | Real Groq calls through production's `TRANSCRIBE_API_KEY` (shared free-tier quota with live users) | **Yes, as planned**: about 5,400 s, at most 3,000 s in one hour. The proxy reads Groq's rate headers and stops if the remaining hourly or daily audio falls under 25% |
| A-4 | Disk (66 GB free at the start) | `cargo clean -p` first, as §4.10. **If free space falls under 15 GB, the whole `target/` may be deleted** (build cache only) |
| A-5 | Permissions | Narrow allow rules in `.claude/settings.local.json` (excluded through `.git/info/exclude`, never committed) for pushing `v6.1-build`, the tag `v6.1` and `revert-v6.1`, `gh pr create/ready/merge --merge` and `gh run rerun`; pushes to `main`, force-pushes and deletes are denied. Removed at the close |
| A-6 | Notifications | A phone notification at key moments only: before the full media run (GPU busy for about 40 min), merged and tagged, any rollback, any stop that needs the founder |
| A-7 | The founder using the laptop in the day | Builds and tests keep running. A media case that fails only under load is re-run once before it counts as a failure, and the flake is recorded |
