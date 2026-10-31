# v5changelog.md - record of every change made while building V5

**What this is.** A record of every change made while building V5: what was done, what differs from `v5implementation.md`, how it was checked, and what is still open. Entries are short (D-195) and in date order, oldest first. Every commit that changes the repository carries its entry. The two tables below are rewritten when an item opens or closes.

**References.** `PS §n` is a section of `product.md`, `TS §n` of `technicalspec.md`, `BP §n` of `buildplan.md`, `V4 CL` is `docs/v4/v4changelog.md`.

---

## Open items
| # | Item | Owner | Version |
|---|---|---|---|
| 1 | **Push the `v4` tag.** The founder said on 2026-10-31 that V4 is closed; on that date `git ls-remote --tags origin` listed `v2` and `v3` only. V5 §1A box 1 waits for it | Founder | Before P129 |
| 2 | **V4's closing entry and boxes.** `v4changelog.md` ends at Entry 39 and `v4implementation.md` has 35 unticked boxes. The agent writes the entry and ticks what the founder vouches for once the tag exists; a box nobody can vouch for stays unticked and is named | Agent | Setup |
| 3 | **Open a draft pull request `v5-build` → `main`** once the branch is pushed: `ci.yml` runs on pull requests | Founder | Setup |

## Known issues
| # | Issue | Where | Version |
|---|---|---|---|
| 1 | **An export reads about 3 LU over the target.** The chain measures its mono programme as one channel, and the export doubles it to two. The same PCM read -23.2 LUFS as one channel and -20.2 LUFS as two (`ffmpeg ebur128`, 2026-10-31), so an export should read about -11 LUFS, and check 7 would fail. D-204 fixes the product | `offcut-dsp/src/loudness.rs` | P130 |
| 2 | V4 CL known issue 33: a flake of `model-download.spec.ts`'s analytics poll on the hosted runner | `web/tests-e2e/model-download.spec.ts` | P129 |
| 3 | V4 CL known issue 36: `openEditor`'s 30 s wait on the hosted runner | `web/tests-e2e/model-download.spec.ts` | P129 |
| 4 | V4 CL known issue 37: re-read `detect_scene` on a quiet D1 | `bench/results/` | P136 |

---

## Entry 0 - 2026-10-31 - `v5implementation.md` written (outside a prompt)

**What.** The plan for V5, at the founder's request of 2026-10-31, after reading BP §7, TS §12, §21, §27, §28, §40.9, V4's plan, changelog and hand-over, and the code at `0a8aa80`. One document holds the plan and the eight prompts (129 to 136), so V5 has no separate build guide or prompts file (D-195). The founder approved the speed rules and the five-run bench on the same day.

**Files.** `docs/v5/v5implementation.md` and this file are new. `docs/technicalspec.md` §5: three rows for `docs/v5/`.

**Found while reading, for the prompts named.** Known issue 1 above (P130). The cache key must use the worker's verified profile: with the page's unverified `planOf`, an expired Creator token would hit a stored 1080p file with no watermark (D-220, P134). `RENDERER_VERSION` exists nowhere in the code (D-223, P134). `export_done`'s `events_kept` counts a switched-off visual as kept (D-237, P135). Every fixture is encoded with `-bf 0`, so `ctts` needs a new one (D-212, P131). Dropping the same file again would hit the stored export unless the import clears `renderCache` (D-228, P134).

**Checked.** Documents only: nothing was built and no test was run. The loudness reading above is the one measurement. Not committed: this session was asked for the document only, and `v5-build` does not exist yet.

**Open.** Open items 1 to 3; known issues 1 to 4.

## Entry 0.1 - 2026-10-31 - `v5buildguide.md` and `coding-promptsv5.md` written (outside a prompt)

**What.** At the founder's request of the same day, after they said V4 is done: a build guide in `v4buildguide.md`'s structure (Phases 0 to 8, steps, a milestone per phase) and the prompts in `coding-promptsv4.md`'s structure (a Setup and 129 to 136). This replaces Entry 0's "no separate build guide or prompts file": D-195 is rewritten, and `v5implementation.md` §22 is now a one-table summary. The speed rules stand. Each phase is one prompt, and the guide's table "The problems V5 must fix" maps Entry 0's six findings and open items 1 to 3 to a step and a proof.

**Files.** `docs/v5/v5buildguide.md`, `docs/v5/coding-promptsv5.md` are new. `docs/v5/v5implementation.md`: the plan's corrections below. `docs/technicalspec.md` §5: two more rows for `docs/v5/`, and one reworded.

**Found while reading the code for these, and applied to the plan (§25 rows 20 to 25).**
- `a_1khz_sine_of_peak_minus_20_reads_minus_23` in `loudness.rs` breaks under the stereo fix, and `pink_noise_reaches_the_target` may. The plan said the tests run unchanged. Both now have a stated rewrite (§8, P130).
- With a shared clip, a reset's undo steps reach the next case through the 10 s analytics flush and fail its `toEqual`. `flushAnalytics` needs `page.clock`, which that page does not install. `resetClip` now sends the queue by `visibilitychange` (§6.3, P129).
- `undo` has no test id; the reset finds it by role. The clean state reads the stored edit rather than opening the Style tab.
- `check-hosts.mjs`'s `DEV_MARKERS` is an explicit list, so D-218's "already fails" was wrong: `devFailEncoder` joins it (P132).
- `sourceHashPrefix` is in the clip's record, not the clip store; a miss's "cached bytes" had no reader, so `heldBytes` is added; the answered set moves to `export-store.ts` so no new use-case edge is needed; `fakeExportDone` is added for the done capture.
- A `FAIL 11` names a tag's key, never its value: `testclips/v3testclip.mp4` carries a location.
- Checked and unaffected: the recognizer hears `pcm16` from the media worker, not the chain, and prosody is z-scored, so the stereo fix moves no transcript or event. `main` has no branch protection, so a skipped CI job blocks nothing.

**Checked.** Documents only: nothing was built and no test was run. `git ls-remote --tags origin v4` printed nothing on this date, so §1A box 1 stays open. Not committed: `v5-build` does not exist yet.

**Open.** Open items 1 to 3; known issues 1 to 4.

## Entry 1 - 2026-10-31 - Setup, first part: branch and baseline

**What.** `v5-build` from `origin/main`, holding the V5 documents and the TS §5 rows, pushed so the founder can open the draft pull request. **Differs from the guide:** Setup is two commits, not one. V4's closing entry waits for the `v4` tag and the founder's answers (open items 1 and 2), and the branch did not need to wait for them. Prompt 129 still does not start until both are done.

| Item | Value |
|---|---|
| Code baseline | `0a8aa80` |
| `pnpm check`, `pnpm fixtures`, `pnpm build` | Exit 0 |
| Rust | 646 passed, 0 failed, 4 ignored |
| Vitest | 175 passed |
| `pnpm e2e` | 12 passed |
| `pnpm e2e:media` | 77 passed + 1 skip, carried from V4 CL Entry 39: the code is the merge's |
| App shell | 337.3 kB gzip |
| `verify_mp4.py`'s tools | NumPy 2.5.3 on Python 3.13, ffmpeg 9.0.2 (§1A box 6) |

**V4's 35 open boxes, checked against the records on this date, for Entry 40.** 28 are shown by evidence and can be ticked when Entry 40 is written: the final gate of V4 CL Entry 39, this baseline, run 40889889605 (all four jobs, with the header check and `@smoke`, which reads `crossOriginIsolated`), `healthz` answering `0a8aa80`, `experiments.md` (14 decisions, none empty), 16 goldens, no untagged `TODO`, no `fetch(` outside the four files. `line_breaks.rs`'s rows are unchanged since `01f406f`: its diff removes only the header comment, an import and the `#[ignore]`. **7 wait for the founder:** `v4implementation.md` lines 1568 (tag; three styles on the deployed page), 1603 (the editor matches the reviewed screens), 1620 (the deployed walk-through), 1621 (the Neon rows), 1626 (design review 2 and five goldens; review 1 is recorded), 1630 (follows 1568) and 1631 (tag).

**Open.** Open items 1 to 3; known issues 1 to 4.
