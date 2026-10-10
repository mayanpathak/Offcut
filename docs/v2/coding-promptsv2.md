# Offcut V2 — coding-promptsv2.md (Prompts 31–60)

Source documents: `docs/v2/v2implementation.md` (written **§n**; its decisions are **D-n**) and `docs/v2/v2buildguide.md` (written **G n.m** for Step n.m, **G M-n** for Milestone n, **G item n** for an item of its "Read this first"). `TS §n` = `technicalspec.md`, `PS §n` = `product.md`, `BP §n` = `buildplan.md`, `V1 §n` = `docs/v1/v1implementation.md`. The prompts were written from the two V2 documents and checked against the three specs, the V1 plan and the repository at `322c7d3`; the corrections to the first draft of this file are under "Revision notes" at the end.

**How to use.** Paste the **Standard Agent Block (SAB)** below, then one prompt, into your coding agent. Run the prompts in order. Do not start Prompt N until every "Done when" box of Prompt N-1 is ticked, and commit after each prompt. Steps marked **Human** are yours, not the agent's (see "Who does what"). V1 ended at Prompt 30; V2 is Prompts 31 to 60; V3 starts at 61.

**Recorded deviation from TS §34.1 and BP §1.3, extended to V2.** Those sections give each file its own specification and hide the rest of the repository from the implementer. `docs/v1/coding-prompts.md` accepted the multi-file format for V1 only. Using it again here is the founder's choice for V2. The per-file contract is still honoured in one way: every prompt names its files, and a signature the plan gives is implemented exactly.

**Two prompts are gates.** Prompt 44 (transcription time) and Prompt 51 (render and encode time) each end with a reading on the reference machine. A miss sends you to §24.3 before anything else is built. Prompt 44's reading was given by the founder on 2026-10-09 (D-67): that prompt takes no timing on R1. **Since D-69 (2026-10-09) the reference machine of V2 is D1, the development machine:** R1 is not available. Every "R1" in Prompts 51, 59 and 60 is read as D1, the agent takes the readings there, and a reading is recorded under the name `d1`.

---

# Standard Agent Block (SAB) — prepend to every prompt

**Every terminal (Git Bash)**

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:$PATH"   # python = 3.13 with NumPy, not the MSYS2 3.9
BASE=$(git merge-base main HEAD)                                             # the V1 baseline every frozen-file check compares with
```

**Before modifying anything**

1. Read the plan sections and guide steps this prompt cites, and the files it touches.
2. Search for existing types, helpers and constants before creating any. Reuse them.
3. Verify that the previous prompts' outputs exist and behave as this prompt assumes. If not: decide whether it is a bug or an intended boundary, make the smallest compatible fix, and report it.
4. Precedence: a decision D-18 to D-71 of §2 is binding, also where it corrects a TS section (it says which). Otherwise TS wins over the plan, and PS over both. The guide gives the order of work, and its "Read this first" items 1 to 12 and its table (b) correct the plan where they say so. If two sources disagree in a way none of these settles, or a signature is missing, stop and list it. Do not guess (TS §34.1).

**During implementation**

1. Implement only this prompt's scope. Create only the files it names. A file that is not in the TS §5 tree fails `check-file-tree.mjs`; if a prompt adds one, it says so, and TS §5 and §4 change in the same commit.
2. Frozen V1 files are not edited: `server/**`, `crates/offcut-types/**`, `crates/offcut-api-types/**`, `web/src/gen/**`, `web/vercel.json`, `render.yaml`, `clippy.toml`, `workers/protocol.ts`, `persistence/schema.ts`, `persistence/db.ts`, `net/http.ts`, `net/api-client.ts`, `analytics/client.ts`, `platform/**`, `state/capability-store.ts`, `state/machines/transition.ts`. A prompt that edits another V1 contract says **REOPENED V1 CONTRACT** and names the decision.
3. No source file over 400 lines. No `unwrap`/`expect`/`panic!`/slice indexing in non-test Rust. No `any`/`!`/`@ts-ignore`/empty `catch` in TypeScript. No disabled lint, no weakened or deleted test, no `eslint-disable` or `#[allow]` to pass a gate. No dependency this prompt does not name; pin each new one at its latest stable release and record the version in `docs/v2/v2changelog.md` (§3.2).
4. V2 has exactly these test files: `crates/offcut-mp4/tests/mux_roundtrip.rs`, `web/src/models/download.test.ts`, and under `web/tests-e2e/` the three media suites and `media.setup.ts`. Every other Rust test is an inline `#[cfg(test)]` module; no other `*.test.ts` is created (§23.1).
5. Use Git Bash. Scripts are POSIX `sh`, invoked with `sh`, never `bash`.
6. A temporary file is named `zz-…` or `zz_…` and is deleted before the gate below. Nothing temporary is committed.
7. A TODO carries the version that removes it (`V3:`, `V7:`). The file name of a dropped `File` is never read. `fetch` exists in the four allowed files only. User-facing text exists in `copy/messages.ts` only.
8. Secrets: no seed, token or connection string is printed, logged or written to a file. `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` is exported in the terminal that needs it and is in no `.env*` file. Production values are in `.env.deploy`; `DATABASE_URL` is the local database.
9. Do not perform a step marked **Human**. List it in your reply.

**Browser checks.** Where a prompt says **Browser check**, run the guide's console snippet yourself, through a temporary Playwright spec `web/tests-e2e/zz-<name>.spec.ts` (`channel: "chrome"`), and print the values it reads:

- "dev" means against `pnpm dev` (start it in the background; `http://localhost:5173`; modules are reachable with `import("/src/…")`). "preview" means against `vite preview` of a production build (`http://localhost:4173`, the production CSP); there the workers are reached through the temporary hook `web/src/zz-spike.ts` of G 5.5.
- Give the page the reference clip without a file picker: `page.route("**/zz-clip.mp4", (r) => r.fulfill({ path: <testclips/speech_scriptA_landscape_720p.mp4> }))`, then in the page `new File([await (await fetch("/zz-clip.mp4")).blob()], "clip.mp4")`.
- A check that needs the speech model uses a persistent Chrome profile under `fixtures/.cache/zz-profile-<port>/`, so the model is downloaded once per origin.
- If Chrome under Playwright lacks WebGPU or an encoder, run headed. If it still cannot, stop and hand the guide's snippet to the human; do not skip the check.

**After implementation: the gate.** Every prompt ends with all of it, in this order.

```bash
git status --porcelain | grep -c "zz[-_]"                    # 0: nothing temporary is left
env | grep -c VITE_ENTITLEMENT_TEST_PUBLIC_KEY               # 0: the gate build carries no test key
pnpm check && pnpm test && pnpm build && pnpm e2e            # all four green
git diff --stat "$BASE" -- server crates/offcut-types crates/offcut-api-types web/src/gen web/vercel.json render.yaml clippy.toml \
  web/src/workers/protocol.ts web/src/persistence/schema.ts web/src/persistence/db.ts web/src/net/http.ts web/src/net/api-client.ts \
  web/src/analytics/client.ts web/src/platform web/src/state/capability-store.ts web/src/state/machines/transition.ts   # (no output)
```

1. Read the test counts from the output: Rust tests passed, Vitest tests passed, Playwright cases passed. None may be lower than in the previous entry of `docs/v2/v2changelog.md`. The baseline is 170, 76 and 12.
2. Fix every regression you caused. Tick each "Done when" box with its evidence. Leave a box that depends on a **Human** step unticked and say so.
3. Append an entry to `docs/v2/v2changelog.md`: what was added and changed, what differs from the plan or the guide, how it was checked (the three counts), pinned versions, what is open. Commit as `V2 P<nn>: <title>`, the changelog entry in the same commit.
4. Reply with: files created and modified, commands run with their results, deviations, **Human** steps still open, and issues found but not fixed.

---

# Who does what

The agent writes files, runs commands and drives Chrome through Playwright. A dashboard, a credential, another machine, a judgement by eye or ear, a push and a merge are **Human** steps.

| When | Human step | Guide |
| --- | --- | --- |
| Before Prompt 31 | Install `gh` and run `gh auth login` (or plan to use the GitHub website). Add the origin `http://localhost:4173` to the asset bucket's CORS policy. Decide whether to replace `ENTITLEMENT_SIGNING_KEY` in Render (its private half was exposed in a chat on 2026-10-08); if you replace it, tell the agent, so that Prompt 33 derives the public half again. Submit the merchant onboarding (TE-9) if it is not submitted. Book R1 and set it up | G 0.1, 0.3, 0.4, 0.6 |
| Prompt 33, and after every prompt marked **Push** | `git push` (the first time `git push -u origin v2-build`, then open the draft pull request). Read the `ci` run; the next prompt waits for green | G M-1, G item 9 |
| Prompt 38 | Choose and download the model files; run the uploads with the credentials of `.env.deploy`; or tell the agent in that prompt's message to do both | G 0.4, 4.2 |
| Prompt 44 | None on R1: the founder's reading of E-3 stands (D-67). Push; read `ci` | G 5.6 |
| Prompt 51 | Nothing on a second machine: TE-3, TE-4 and the three export timings are read on D1 by the agent (D-69). Push; read `ci` | G 8.4, 8.5 |
| Prompt 55 | Watch and listen to one preview and one exported file: caption position, sound in sync, the reveal on the spoken amount | G 10.4, 10.5 |
| Prompt 58 | Mark the pull request ready; read the Windows job and its minutes (TE-10) | G 11.5 |
| Prompt 59 | Merge to `main`; check the deployed page on D1; read the production analytics rows; the bench, `pnpm e2e:device` and TE-14 run on D1 (D-69) | G 12.2 to 12.5 |
| Prompt 60 | Read E-1 from the production database; take the M0 decision; push the tag | G 12.7 |
| Before the page is announced | Reset the Neon password (V1 open item 36); replace the demo video (V1 known issue 33) | G 0.4 |

---

# Project Implementation Map

## Architecture summary

V2 is the proof of concept: the reference clip goes in, and a 1080x1920 MP4 with Clean captions and one NumberReveal comes out, entirely in the browser and deployed. Three workers (`media`, `asr`, `render`) sit behind typed clients. Two WASM bundles hold every rule that affects output: `offcut_core.wasm` (demux, probe, validate, resample, hash, text) and `offcut_render.wasm` (detect, scene, GPU render, mux, token to profile). The server is not touched. V3 and V4 must be additive: later versions fill bodies and add files and rows (§25).

## Major components

| Component | Location |
| --- | --- |
| MP4/MOV demux, probe, validate, faststart mux | `crates/offcut-mp4` |
| Resampler | `crates/offcut-dsp` |
| Tokens, sentences, numbers (V2 subset) | `crates/offcut-text` |
| NumberReveal detection, event ids | `crates/offcut-detect` |
| Token decode and verify, `ExportProfile` | `crates/offcut-entitlement` |
| Clean captions, NumberReveal, framing, display list | `crates/offcut-scene` |
| wgpu + Vello compositor | `crates/offcut-render` |
| Bindings | `crates/offcut-wasm-core`, `crates/offcut-wasm-render` |
| OPFS paths, RPC, pool, three workers | `web/src/persistence/opfs.ts`, `web/src/workers/**` |
| Model manifest, download, verify | `web/src/config/model-manifest.json`, `web/src/models/**` |
| Machines, stores, blockers, use-cases, UI | `web/src/state/**`, `web/src/usecases/**`, `web/src/ui/**` |
| Verifier, bench, media E2E, CI | `verify/`, `bench/`, `web/tests-e2e/`, `.github/workflows/` |

## Dependency graph

```text
preflight, docs, D-65, crate skeletons → tree/ban/lint rules → config keys, worker probe
        │
        ├─ offcut-mp4 read ──► core bindings ──► opfs + rpc + pool + media worker (ingest)
        │        │                                        │
        │        └─► muxer                    model upload + manifest → downloader → manager + panel
        │                                                 │
        ├─ offcut-text ──► ASR worker ──► TE-1, TE-2, E-3  [GATE]
        │        │
        │        ├─► offcut-detect ─┐
        │        └─► offcut-scene ──┼─► offcut-render ─► wasm-render bundle
        │   offcut-entitlement ─────┘                          │
        │                           verifier, encoders, sink, video source → export loop + render worker
        │                                                      │
        │                                   spike export → TE-3, TE-4, E-4  [GATE]
        ▼
machines, stores, blockers → preview loop + control-preview → use-cases → UI and landing navigation
        → media E2E (three suites) → bench + CI + pull request → deploy + measure → M0 gate, tag v2
```

## Phases

| Guide phase | Prompts |
| --- | --- |
| 0 Environment, 1 Contracts and setup | 31–33 |
| 2 MP4 read side | 34–35 |
| 3 Resampler, bindings, workers, ingest | 36–37 |
| 4 Model delivery and muxer | 38–41 |
| 5 ASR spike (gate) | 42–44 |
| 6 Scene, detection, entitlement | 45–46 |
| 7 Renderer and render bindings | 47–48 |
| 8 Render and encode spike (gate) | 49–51 |
| 9 State and use-cases | 52–54 |
| 10 UI and copy | 55 |
| 11 E2E, CI, bench | 56–58 |
| 12 Deploy, measure, gate, tag | 59–60 |

## Prompt dependency table

Run in order. Allowed in parallel: Prompt 41 needs only 35; Prompts 45 and 46 need only 42; server work for V6 depends only on V1 and may fill a wait (BP §0).

| Prompt | Major deliverable | Depends on | Produces | Guide |
| --- | --- | --- | --- | --- |
| 31 | Preflight, branch, V2 docs, D-65, eight crate skeletons | V1 on `main` | 12 workspace members; baseline recorded | Phase 0, 1.1–1.2 |
| 32 | Tree, purity and ban rules; test tooling; lint edges | 31 | The rules V2 code is judged by | 1.3–1.5 |
| 33 | Config keys, clip check, worker probe | 32 | Proof that a worker gets WebGPU, WebCodecs, sync OPFS under the CSP; draft PR. **Push** | 1.6–1.8, M-1 |
| 34 | `offcut-mp4`: errors, reader, boxes, sample tables | 33 | Parsed tables | 2.1, 2.2 |
| 35 | `offcut-mp4`: demux, probe, validate | 34 | `ProbeInfo`, `ClipInfo`, 11 rules. **Push** | 2.2, 2.3, M-2 |
| 36 | Resampler, core bindings, `CoreApi` | 35 | `offcut_core.wasm` with media and hash exports | 3.1–3.3 |
| 37 | `opfs.ts`, `rpc.ts`, `pool.ts`, media worker | 36 | Clip in OPFS; exact-length PCM. **Push** | 3.4–3.6, M-3 |
| 38 | Upload script, model on the asset host, manifest, `fetchAsset` | 37 | Hashed manifest; sample clip path | 4.1–4.3 |
| 39 | Model machine, store, downloader, `download.test.ts` | 38 | Resumable, verified download | 4.4 |
| 40 | Model manager, panel, copy | 39 | Model reaches `ready` and survives a reload | 4.4, 4.5 |
| 41 | MP4 muxer, `mux_roundtrip.rs` | 35 | Faststart MP4 writer. **Push** | 4.6, M-4 |
| 42 | `offcut-text` and its binding | 36 | `Transcript` from raw words | 5.1 |
| 43 | ORT copy step, ASR worker, host entries, first transcript | 40, 42 | Word-timed transcript in dev | 5.2–5.5 |
| 44 | TE-1, TE-2, first E-3 **(gate)** | 43 | Recorded outcomes; go or stop. **Push** | 5.5, 5.6, M-5 |
| 45 | `offcut-scene` | 42 | Display list per frame | 6.1, 6.2 |
| 46 | `offcut-detect`, `offcut-entitlement`, token vector | 42 | Events; verified token to profile. **Push** | 6.3–6.5, M-6 |
| 47 | `offcut-render` | 45 | Compositor on wasm32 | 7.1, 7.2 |
| 48 | `offcut-wasm-render`, bundle, loader, preload | 41, 46, 47 | `offcut_render.wasm`. **Push** | 7.3–7.5, M-7 |
| 49 | Verifier, encoders, sink, video source | 48 | Checks 1–6; export building blocks | 8.1, 8.2 |
| 50 | Export loop, render worker, first export | 49 | An MP4 the verifier reads | 8.2, 8.3 |
| 51 | TE-3, TE-4, first E-4 **(gate)** | 50 | Capture method, ladder, priming; go or stop. **Push** | 8.3–8.5, M-8 |
| 52 | Machines, stores, blockers, entitlement repo | 40 | State layer | 9.1–9.3 |
| 53 | Preview loop, preview handlers, `control-preview` | 50, 52 | Playing preview | 9.4, 9.5 |
| 54 | `run-pipeline`, `import-clip`, `start-export`, `start-app` | 53 | Drop to `ready`; seeded export. **Push** | 9.5, M-9 |
| 55 | Copy, components, pages, landing navigation | 54 | The one path by hand. **Push** | 10.1–10.5, M-10 |
| 56 | E2E helpers, setup project, `model-download.spec.ts` | 55 | 8 media cases | 11.1, 11.2 |
| 57 | `pipeline-preview.spec.ts`, `export-creator.spec.ts` | 56 | 30 media cases | 11.2 |
| 58 | Bench, `e2e-media.yml`, CI changes, test-key guard | 57 | Green pull request; TE-10. **Push** | 11.3–11.5, M-11 |
| 59 | Deploy and measure | 58 | Live V2; R1 results; TE-14 | 12.1–12.5 |
| 60 | Records, spec corrections, M0 gate, tag `v2` | 59 | Tagged V2 | 12.6, 12.7, M-12 |

---

# Prompts

Each prompt ends with the SAB gate. The "Done when" boxes are what is specific to the prompt.

## Prompt 31 — Preflight, branch, V2 documents, D-65, crate skeletons

**Objective.** Verify the machine and the V1 baseline, then give the repository every V2 crate as an empty member (§1A, §4, §22.1; G Phase 0, 1.1, 1.2). **State before.** `main` at the deployed V1 commit; the V2 documents edited and not committed. **Implement.**

- Run the audit block of G 0.1 and report every version. `python --version` must say 3.13.
- `git switch -c v2-build`. Commit the pending documents alone (`docs/v2/*.md`, `docs/v1/v1changelog.md`, `fixtures/speech/README.md`) as `V2: documents`. Then `pnpm install --frozen-lockfile` and the four-command gate. Record `git rev-parse main` as the baseline SHA.
- The checks of G 0.3 that need no dashboard: `tsc --noEmit`; the four encoder names in `encoders.ts`; `bench/results` exists; "twelve thousand dollars" is in the README; the public key of §1A item 12 decodes to 32 bytes; `/api/v1/healthz` on both hosts reports `main`; `node scripts/check-headers.mjs <app origin>`; a ranged `curl` to an uploaded asset answers 206; the `curl` of G 0.4 for the origin `http://localhost:4173`; `E2E_BASE_URL=<app origin> pnpm --filter web exec playwright test --project=non-media --grep @smoke`.
- Create `docs/v2/experiments.md` and `docs/v2/v2changelog.md`. First changelog entry: baseline SHA, tool versions, the three test counts.
- TS §5 tree: add `crates/offcut-mp4/src/mux_boxes.rs`, `web/tests-e2e/tsconfig.json` and the six files of `docs/v2/` (`v2implementation.md`, `v2implementation-notes.md`, `v2buildguide.md`, `coding-promptsv2.md`, `experiments.md`, `v2changelog.md`). Extend D-46 to the same six.
- D-65 in §2 of the plan (R1 is the only reference machine; reason and cost from G item 12) and the reworded lines of the table in G 1.1: §0 and §2 ("D-18 to D-65"), §1A boxes 13 and 14, §5 S15, §23.1, §24.1 (E-3, TE-3, TE-4, E-4), §25.3, §26.
- Eight skeletons, directories before the manifest (G 1.2): `crates/offcut-{mp4,dsp,text,detect,entitlement,scene,render,wasm-render}` with an empty `src/lib.rs`, the `[package]` style of `offcut-types`, `[lints] workspace = true`; `offcut-wasm-render` is `cdylib`; `offcut-entitlement` declares `[features] sign = []`. Then the eight members in the root `Cargo.toml`.

**Not yet.** Any dependency, any logic, any rule change (Prompt 32). **Human, before this prompt.** The first row of "Who does what". Report each item that is still open; the CORS origin is needed by Prompt 44 at the latest. **Done when.**

- [ ] Every tool prints its version; the baseline gate passed on the branch before any V2 change; the baseline SHA is in `v2changelog.md`.
- [ ] `cargo metadata --no-deps --format-version 1` lists 12 packages.
- [ ] `grep -nw "R2" docs/v2/v2implementation.md` shows only lines that say R2 is not used; no box still asks for it.
- [ ] Every item of §1A is ticked or reported open with its owner.

## Prompt 32 — Tree, purity and ban rules; test tooling; lint edges

**Objective.** Put in place every rule V2 code will be judged by, before any V2 code exists (G 1.3–1.5; D-27, D-28, D-58, D-59, D-63). **REOPENED V1 CONTRACT:** `web/eslint.config.js`. **Implement.**

- `scripts/check-file-tree.mjs`: `ALLOWED_CRATE_DEPS["offcut-scene"]` gains `offcut-text` (D-28); `PURE_CRATES` gains `offcut-mp4`, `-dsp`, `-text`, `-detect`, `-scene`, `-entitlement`.
- `deny.toml`: the `offcut-text` entry gains the wrapper `offcut-scene`. Nothing else yet.
- `.gitignore`: `web/public/ort/`, `fixtures/.cache/`.
- D-63: root dev-dependency `@playwright/test` at exactly the version `pnpm-lock.yaml` resolves for `web` (1.63.0 today); `@types/node` at the Node major of `engines` as a dev-dependency of `web`; new `web/tests-e2e/tsconfig.json` (extends `../tsconfig.json`, adds `node` to `types`, includes `tests-e2e`, `playwright.config.ts`, `../bench`); `tests-e2e` leaves the `include` of `web/tsconfig.json`; the root `check` script runs a second `tsc --noEmit -p tests-e2e/tsconfig.json`.
- `web/eslint.config.js`, exactly 15 entries and no existing rule loosened: the six edges of D-27 (a to f; f is type-only), the two use-case pairs of D-58, the seven cast overrides of D-59. A cast override must not switch the rule off for its file: it exempts one named minting helper (or, in `model-store.ts`, one named constant) and still rejects a second cast in the same file. Name the seven in `v2changelog.md`.

**Validate (each must fail, then revert).** The four rows of G 1.3 (`zz.rs` in `offcut-mp4`; `offcut-text` as a dependency of `offcut-mp4`; `rand` in `offcut-entitlement`; `web-sys` in `offcut-scene`). The two rows of G 1.4 (`process.cwd()` fails `tsc` in `web/src/main.tsx` and passes in `tests-e2e`). The three rows of G 1.5 (a brand cast in a `ui/` file; a type import of `workers/protocol` in `ui/`; `persistence/opfs` imported from `state/`). **Done when.**

- [ ] All nine drills fired and were reverted; `git check-ignore web/public/ort/x fixtures/.cache/x testclips/x` echoes three paths.
- [ ] The ESLint diff holds 15 new entries; `pnpm check` runs two `tsc` passes.

## Prompt 33 — Config contracts, reference clip, worker probe

**Objective.** The two config files V2 code imports, and proof under the production CSP that a module worker gets WebGPU on an `OffscreenCanvas`, H.264 and AAC encoders and a sync OPFS handle (§15.1, D-24, D-39, D-64; G 1.6–1.8). **Implement.**

- `web/src/config/env.ts`: one new field `entitlementTestPublicKey` (`string | null`); a set value that does not decode to 32 bytes throws at module load.
- `web/src/config/entitlement-public-key.ts`: `ENTITLEMENT_PUBLIC_KEYS`, the production key of §1A item 12 as a base64 literal, plus the test key only when the variable is set. If the human replaced the key in Render, derive the public half again first (G 0.3) and correct §1A item 12; never write a seed anywhere.
- The reference clip: the three commands of G 1.7. Write the video-stream duration in milliseconds into `fixtures/speech/README.md`.
- The probe of G 1.8, temporary: `web/src/workers/zz-probe.worker.ts` and one line in `main.tsx`; `pnpm build`, then read the console line through a browser check on preview. Remove both the same day.

**Browser check (dev).** `ENTITLEMENT_PUBLIC_KEYS.map((k) => k.length)` gives `[32]`; with `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` set to any base64 of 32 random bytes it gives `[32, 32]`; with `abc` the import rejects. **Gate.** A `false` or an error key in the probe: stop, record it in `experiments.md`, read the TS §19 contingency (WebGPU) or TE-4 (encoders). **Human.** `git push -u origin v2-build`; open the draft pull request; read `ci`. If the secret scan names the public key line, the agent ends it with `// gitleaks:allow` (G item 9). **Done when.**

- [ ] The probe printed `webgpuCanvas`, `h264`, `aac`, `opfsSync` all `true` under `vite preview`, with no line starting "Refused to".
- [ ] The clip reads 1280x720, H.264 and AAC, 48 kHz stereo, 74,705 ms, at most 40,000,000 bytes, two white frames (frames 150 and 1950, `YAVG` at least 230).
- [ ] `git grep -n "zz-probe" -- web` is empty; G M-1 passes; the draft pull request is open and `ci` is green (Human).

## Prompt 34 — `offcut-mp4`: errors, reader, boxes, sample tables

**Objective.** Read the structure of an MP4/MOV through `RandomAccess`, with every count bounded (§6.1–§6.4, D-60; G 2.1, 2.2 rows 1–4). **Implement.**

- `Cargo.toml`: `offcut-types`, `thiserror`; dev `proptest` (new: pin and record). If `cargo deny check` stops at `bans`, add the third-party crate it names as a wrapper (G 2.1); never a workspace crate.
- `lib.rs`: `IoError` (3 variants), `ContainerError` (6), `MuxError` (4), exactly as §6.1. `Malformed` and `Unsupported` carry a box name, never file bytes.
- `reader.rs`: the `RandomAccess` trait of TS §15.1 verbatim; `MemReader`; a read past `len()` is `OutOfBounds`.
- `boxes.rs`: `read_header`, `children`, the size rules and bounds of the §6.3 table (32-bit, 64-bit, zero; depth 16; 4,096 children), and the typed readers it lists. Every table count is bounded by `body length / entry size` before any allocation.
- `sample_table.rs`: `resolve`, steps 1 to 6 of §6.4. A track of more than 20,000 samples is left unresolved; that is not an error.

**Not yet.** `demux.rs`, `probe.rs`, `validate.rs` (Prompt 35); `mux*.rs` (Prompt 41). **Tests (inline).** Seven rows of §6.9: header sizes; a box past the limit; the `stts` sum; `ctts` version 1; no `stss`; one media edit; two media edits. **Validate (must fail, then revert).** `let _ = v[0];` in `boxes.rs` non-test code (clippy `indexing_slicing`). **Done when.**

- [ ] `cargo test -p offcut-mp4 --lib` and `cargo clippy -p offcut-mp4 --all-targets -- -D warnings` pass; the seven rows are covered.
- [ ] `cargo deny check` and `node scripts/check-file-tree.mjs` pass with `offcut-mp4` now checked as a pure crate.

## Prompt 35 — `offcut-mp4`: demux, probe, validate

**Objective.** `ProbeInfo` for any file, and `ClipInfo` or a `RejectReason` from the 11 rules (§6.5–§6.7, D-21, D-34; G 2.2 rows 5–7, 2.3). **Implement.**

- `demux.rs`: `SampleMeta` and the `Demuxer<R>` signatures of TS §15.1 verbatim; `open` steps 1 to 4 (fragmented, not ISO, `moov` after `mdat`, the `t0` time base, floor division to `Micros`); the three method contracts. A file shorter than one 8-byte box header is `NotIsoBmff`, not `Truncated` (the guide's rule; §6.3 and §6.5 leave it open).
- `probe.rs`: every field rule of §6.6. `probe` never fails and never reads the expanded sample list. `duration` is the presented video duration, rounded to the nearest millisecond.
- `validate.rs`: the 11 rules of TS §15.2 verbatim and in that order, first failure wins; the only constructor of `ClipInfo`; limits from `offcut_types::limits` only; it never returns `FileSize`, `Corrupt` or `NoSpeech`.

**Tests (inline).** The other nine rows of §6.9 and the short-file case: 17 in the crate. **Validate (must fail, then revert).** `.unwrap()` in `demux.rs` non-test code; `std::time::Instant::now()` in `probe.rs`. **Real clip (temporary, never committed).** The ignored test `zz_real_clip` of G 2.3. It must print: duration 74,705 within 1; `H264`, a `codec_string` starting `avc1.4d`; 1280, 720, `R0`; `frame_count` 2,246; `is_vfr` true; `Aac`, `mp4a.40.2`, 48,000 Hz, 2 channels; `Ok(ClipInfo)` with `Landscape`. A different value is a demuxer bug: fix it, do not adjust the expectation. Then delete the test. **Human.** Push; read `ci`. **Done when.**

- [ ] `cargo test -p offcut-mp4 --lib`: the 16 rows of §6.9 and the short-file case pass.
- [ ] The real clip printed the values above; `git grep -n zz_real_clip` is empty.
- [ ] `validate_probe` has 11 rejecting branches, read against TS §15.2 line by line; G M-2 passes.

## Prompt 36 — Resampler, core bindings, `CoreApi`

**Objective.** The media and hash exports of `offcut_core.wasm` (§7, §13.1–§13.3, D-31, D-36, D-53; G 3.1–3.3). **Implement.**

- `offcut-dsp`: `Cargo.toml` (`offcut-types`, `rubato`, `thiserror`; dev `proptest`), `lib.rs` (`pub mod resample;`, `DspError { Empty, NonFinite }`), `resample.rs` (`resample_mono`: output length exactly `round(input.len() x to / from)`, group delay trimmed, same input gives identical output).
- `offcut-wasm-core`: add `offcut-mp4`, `offcut-dsp`, `sha2`, `serde-wasm-bindgen`, `js-sys`, `web-sys`; not `offcut-text` yet. `lib.rs`: two module declarations and the `{ code, detail }` helper (`detail` is a variant name only). `hash_api.rs`: `Sha256Stream`, lower-case hex. `media_api.rs`: the signatures of §13.1 and the shapes of §13.2; a rejection is a returned value; a short read is `IoError::Read`; conversion through `Serializer::json_compatible()`.
- `web/src/wasm/load-core.ts`: `CoreApi` of §13.3 without `normalizeTranscript` (Prompt 42). A thrown `{ rejected }` is returned as a value; a thrown `{ code, detail }` is rethrown unchanged. Correct the comment on `loadCore()` (D-36).

**Tests (inline).** The four rows of §7. **Validate.** After `cargo check -p offcut-wasm-core --target wasm32-unknown-unknown`: the `wasm-bindgen` version in `Cargo.lock` equals `wasm-bindgen --version`. The workspace pins it exactly, so a resolution error means a new dependency wants a newer one: follow the second row of G 3.2, do not loosen the pin. **Browser check (dev).** The SHA-256 of `"abc"` is `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`; `core.resample(new Float32Array(48000), 48000, 16000).length` is 16000. **Done when.**

- [ ] `cargo test -p offcut-dsp` passes with its proptest; `pnpm build:wasm` writes `web/src/wasm/pkg/core/`.
- [ ] `pnpm gen:types && sh scripts/check-gen-clean.sh` shows no diff; both browser values match.

## Prompt 37 — `opfs.ts`, `rpc.ts`, `pool.ts`, media worker

**Objective.** The browser copies a file to OPFS, probes and validates it in Rust inside `media.worker`, and returns PCM of exactly the right length (§15.2, §15.6, §15.7, §16.1–§16.3, D-32, D-34; G 3.4–3.6). **Implement.**

- `persistence/opfs.ts`: the ten `paths` functions (TS §23.1 verbatim) and the helpers of §15.2. `modelFile` and `modelPart` reject a name containing `/` or `..`. `QuotaExceededError` is rethrown tagged `quota`, any other failure `io`. One minting helper for `Bytes`.
- `workers/rpc.ts`: `createClient`, `serveWorker`, `toAppFailure`, `CANCELLED`, the two behaviour tables of §15.6; the "Retryable" column of TS §11.2 as one table here. No throttle, no cancel timeout (V4).
- `workers/pool.ts`: one table row, `media`; lazy creation; `preload()` keeps its V1 signature and gains the `modulepreload` link for the script; re-export `WorkerCallError` and `Cancelled`. Not yet: the `asr` and `render` rows, `preloadRender()` (G item 5).
- `workers/media/import.ts` (4 MiB chunks, cancel check before each, progress per chunk), `media/audio-decode.ts` (every `AudioData` closed in `finally`; alignment by timestamps only), `media.worker.ts` (top level is `serveWorker(...)` only; the size rule before any copy; `importAndProbe` steps 1 to 6; `extractAudio` with the pad or truncate of D-34; the sync handle closed in `finally`).

**Not yet.** `rpc.test.ts` (V4). **Browser check (dev).** The script of G 3.6 on the reference clip: `duration` within 1 of 74,705; `pcm48.length` 3,585,840; `pcm16.length` 1,195,280; OPFS holds `clips/<clipId>/source` and no name from the dropped file. A `File` of 500,000,001 bytes (500 references to one 1 MB buffer plus one byte) gives `{ rejected: "REJECT_FILE_SIZE" }` and creates no directory. A 6-byte text file gives `{ rejected: "REJECT_CONTAINER" }`. `paths.modelFile("m", "../x")` throws; `await size("nope")` is `null`. **Human.** Push; read `ci`. **Done when.**

- [ ] The five numbers and the two rejections match; G M-3 passes.
- [ ] ESLint is clean for `persistence`, `workers`: edge D-27 b proves its positive side; a second `Bytes` cast in `opfs.ts` fails lint.

## Prompt 38 — Upload script, model on the asset host, manifest, `fetchAsset`

**Objective.** The speech model and the sample clip sit on the asset host under hashed names, and the app can fetch a range of one (§15.1, §15.4, D-39, D-62; G 4.1–4.3; TE-7 re-check). **Implement, in this order.**

1. `scripts/upload-assets.sh` (D-62): the folder argument may be `media`, `models` or `models/<modelId>` (one more segment of lower-case letters, digits and hyphens); the content-type table gains a row for each extension of the model's files. Names keep their hash segment.
2. **Human, or the agent when this prompt's message says so.** Model files into a folder outside the repository: the small-size English model (D-66) with word timestamps, an export that returns cross-attention outputs, quantized so that `wc -c` totals at most 260,000,000 bytes (TS §16.1). `sha256sum` of each. Upload each to `models/asr-en-v1` and the reference clip to `media`, credentials from `.env.deploy` for that command only, never printed. Keep every printed path.
3. `web/src/config/model-manifest.json`: the shape of TS §16.1; `modelId` `"asr-en-v1"`; each `path` is a printed path, each `bytes` from `wc -c`, each `sha256` from `sha256sum`, written by a one-off command and never typed by hand; `totalBytes` is their sum. No generator script is added to the tree.
4. `web/src/net/asset-fetch.ts`: `SAMPLE_CLIP_PATH` (the printed path) and `fetchAsset` (the table of §15.4: `credentials: "omit"`, no query, one `Range` header, no retry, no `ErrorCode`).

**Validate (must fail, then revert).** The three bad folder arguments of G 4.1 (`models/../media`, `models/ASR_EN`, `models/asr-en-v1/extra`) exit non-zero with nothing uploaded. **TE-7.** The two `curl` lines of G 4.2 on the largest file: `206 8388608`; the immutable cache header; the three exposed headers. Record it in `experiments.md`. **Browser check (dev).** `fetchAsset(SAMPLE_CLIP_PATH, { range: 0–1023 })` gives `[true, 206]`, with no cookie and no query string; offline it gives `cause: "offline"`. **Note.** Which files the runtime asks for is only certain in Prompt 43. If it asks for one the manifest lacks, repeat steps 2 and 3. **Done when.**

- [ ] The manifest total is at most 260,000,000 and equals the sum of `files[].bytes`; every `path` starts with `models/asr-en-v1/`.
- [ ] TE-7 is recorded; the ranged fetch from the dev page answers 206.

## Prompt 39 — Model machine, store, downloader

**Objective.** A resumable ranged download with hash verification, proven with fakes (§17.2, §17.3, §18.1, INV-20; G 4.4 rows 1–4). **Implement.**

- `state/machines/model-machine.ts`: the 14 (state, event) pairs of §18.1; `verifying` to `downloading` is absent.
- `state/model-store.ts`: the store and its eight actions, every change through `transition()`; one named constant for the `Bytes` zero.
- `models/download.ts`: the constants (8 MiB parts, 3 retries, backoff 1, 3, 9 s), `createDownloader(deps)`, `fetchRanged` (the eight rules of §17.2), `verifyAndFinalize` (4 MiB slices, a yield between slices; a match moves the part to its final name, a mismatch removes it), `localName` (D-62).
- `models/download.test.ts`: the 16 cases of §23.3, the manifest assertion among them.

**Not yet.** `model-manager.ts`, the panel (Prompt 40). **Validate (must fail, then revert).** `import { http } from "../net/http";` in `download.ts` (boundaries). **Done when.**

- [ ] `pnpm --filter web exec vitest run src/models/download.test.ts`: 16 cases pass; the Vitest total is 92 or more.
- [ ] ESLint is clean for `src/models` and `src/state`: edges D-27 d and f prove their positive side.

## Prompt 40 — Model manager, download panel, copy

**Objective.** The model reaches `ready` once and is skipped afterwards (§17.1, §20.2, §20.3, D-36, D-57; G 4.4 row 5, 4.5). **Implement.**

- `models/model-manager.ts`: `inspect`, `ensureReady` (one promise at a time; `navigator.storage.persist()` once, guarded by `META_KEYS.persistRequested`), `cacheInfo`, `clear`; the store events of the §17.3 table. A final name exists only after the hash matched.
- `ui/components/ModelDownloadPanel.tsx` (reads the model store; the size from the manifest rounded to 10 MB; bar width through `style.setProperty`), its classes in `components.module.css`.
- `copy/messages.ts`: `modelDownload` and `errors.E_MODEL_DOWNLOAD`, `E_MODEL_HASH`, `E_MODEL_STORAGE`. `scripts/check-copy-codes.mjs`: those three leave `COPY_PENDING` in this same commit (V1 D-13).

**Validate.** Put `E_MODEL_HASH` back into `COPY_PENDING`: the copy check must fail; revert. Typing `150 MB` into `modelDownload.body`: no tool fires (V1 has none before V8); read the new strings for a digit and note the gap in `v2changelog.md`. **Browser check (dev, persistent profile).** A cold download shows only 206 responses from the asset host and ends `ready`; OPFS `models/asr-en-v1/` holds plain names and no `.part`. After a reload `inspect()` gives `ready` with zero model requests. Close the page at about 30% of a second cold download, reopen, `ensureReady`: the first `Range` starts at the `.part` size. With one hex digit of a manifest `sha256` changed: the store ends `failed` with `E_MODEL_HASH`, and no final file and no `.part` exists for that file; restore the digit. **Done when.**

- [ ] The four browser results hold; `node scripts/check-copy-codes.mjs` passes with three fewer pending codes.
- [ ] Edge D-27 e is used and lints clean; the panel lints and type-checks without being mounted.

## Prompt 41 — MP4 muxer

**Objective.** A faststart MP4 that the crate's own demuxer reads back byte for byte (§6.8, §23.2, D-33, D-55; G 4.6). **Implement.**

- `mux_boxes.rs`: one writer per box and nothing else; no `udta`, no rotation.
- `mux.rs`: `MuxSink`, `VideoTrackSpec`, `AudioTrackSpec`, `Mp4Muxer<S>` of TS §21.1 verbatim; `MemSink`; the four call contracts of §6.8 (`MOOV_RESERVE` 256 KiB; a chunk every 15 video and 24 audio samples; the audio `elst` from a negative first `pts`); each `add_*` writes through the sink at once.
- `tests/mux_roundtrip.rs`: the 12 cases of §23.2, one of them proptest.

**Not yet.** `ctts`, the overflow test (V5). **Second opinion (temporary, never committed).** Write the first round-trip case's bytes to `target/zz_mux.mp4`; `ffprobe` must show `h264` with 90 frames and `aac` with 141, time bases 1/30000 and 1/48000. Remove the line. **Human.** Push; read `ci`. **Done when.**

- [ ] `cargo test -p offcut-mp4`: lib and `mux_roundtrip` pass; box order is `ftyp`, `moov`, `mdat`.
- [ ] `ffprobe` agreed; `git grep -n zz_mux` is empty; G M-4 passes.

## Prompt 42 — `offcut-text` and its binding

**Objective.** A `Transcript` with sentences and number spans from raw words (§8, §13.3, D-31, D-42; G 5.1). **Implement.**

- `offcut-text`: `Cargo.toml` (`offcut-types`, `serde` only); `tokenize.rs` (a leading `$` and a trailing `%` stay on the `Digits` token); `sentences.rs` (`SENTENCE_GAP` 700 ms; `SENTENCE_MAX_WORDS` declared, unused); `numbers.rs` (`parse_quantity` for the four forms of §8.4, `None` for anything else, never a guess; `format_quantity`, the only formatter, with the display rules of §8.4); `normalize.rs` (`RawWord` in camelCase with `deny_unknown_fields`; `normalize_transcript`, the only constructor of a `Transcript`).
- `offcut-wasm-core`: the dependency, `text_api.rs`, the third `mod`. `load-core.ts`: `normalizeTranscript`.

**Not yet.** The conditional `dollars` form of D-42 (Prompt 43 decides); millions, suffixes, units (V3). **Tests (inline).** The 11 unconditional rows of §8.5. **Done when.**

- [ ] `cargo test -p offcut-text` passes the 11 rows; `pnpm build:wasm` and `tsc` are clean.
- [ ] `node scripts/check-file-tree.mjs` and `cargo deny check` accept the edge `offcut-wasm-core` to `offcut-text`.

## Prompt 43 — ORT copy step, ASR worker, host entries, first transcript

**Objective.** `asr.worker` returns word timestamps for the reference clip, loading model files from OPFS only (§16.4, §16.5, §22.3, §22.4, D-37, D-38, D-50; G 5.2–5.5). **REOPENED V1 CONTRACT:** `scripts/check-hosts.mjs` (D-38) and `web/eslint.config.js` (one rule). **Implement.**

- `web/package.json`: `@huggingface/transformers`, exact version. `web/vite.config.ts`: the build-start copy of the ONNX Runtime files into `web/public/ort/<version>/`, resolved through the package that depends on it (the snippet of G 5.2); other version directories are removed.
- `workers/asr/word-timestamps.ts` (`mergeWindows`, `postProcess`; no word is dropped, merged or shifted because of a pause), `asr/model-cache-adapter.ts` (maps the requested base name to `paths.modelFile`; a miss throws, never fetches; reads final names only), `asr/whisper-runtime.ts` (the only importer of the runtime package; remote loading off, the runtime's own cache off, `.wasm` path `/ort/<version>/`, `webgpu` then `wasm` with `clamp(hardwareConcurrency - 2, 1, 4)` threads; 30 s windows with 5 s overlap; `confidence` 1.0 until Prompt 44), `asr.worker.ts` (`load`, `transcribe`, `unload`; `unload` resolves only after the session is disposed), the `asr` row in `pool.ts`.
- `web/eslint.config.js`: V1 has no rule for TDR-3. Add one `no-restricted-imports` rule: `@huggingface/transformers` and `onnxruntime-web` may be imported by `workers/asr/whisper-runtime.ts` only.
- `scripts/check-hosts.mjs`: an entry may carry `chunk`; with it the literal is allowed only in output files whose path matches. One entry per literal the failing build names, each with its reason, for the `asr.worker` chunk and for `ort/<version>/`. The reason says that remote loading is off and that Prompt 44 (TE-1) proves no request.
- `fixtures/speech/README.md`: the transcript of the run below as the expected transcript, and the expected events of V2 (the amounts that must be found).

**Validate (must fail, then revert).** The runtime imported in `asr.worker.ts` (the new rule). One listed host literal pasted into `web/src/main.tsx` (`check-hosts` on the main chunk). One new entry's `chunk` pointed at a pattern that matches nothing (`check-hosts` on the worker chunk). **Browser check (dev, model cached).** The script of G 5.4: about 150 to 190 words; times increasing, the first under 3,000 ms, the last under 74,705; `numbers` holds one `Usd` span for the twelve-thousand-dollar amount with display `$12k`. If the amount arrives as `12,000 dollars` or in words, add the conditional form of §8.4 with its test row now and say so in `v2changelog.md`. Fewer than 5 words, or no times, is a model-export or cache-adapter fault: fix it before the gate. **Done when.**

- [ ] The three drills fired; `pnpm build` passes `check-hosts` with the runtime in `web/dist`; `ls web/dist/ort/` shows one version directory.
- [ ] The transcript carries the dollar amount as a `Usd` quantity; every runtime option name used is in `v2changelog.md`.

## Prompt 44 — TE-1, TE-2, first E-3 (gate)

**Objective.** Answer the first question that can stop the project: does local transcription stay inside the closed network list and inside its time budget (§24.1, §24.3, D-64; G 5.5, 5.6). **Implement.**

- The temporary hook `web/src/zz-spike.ts` and its import in `main.tsx` (G 5.5); a production build; `vite preview`.
- TE-2: if the runtime gives a per-token probability, wire `confidence` as its mean per word in `word-timestamps.ts`.
- `docs/v2/experiments.md`: TE-1, TE-2 and E-3, each with question, method, numbers, date, decision.

**Browser check (preview; needs the CORS origin `http://localhost:4173`).** Record every request of the page and its workers while the model loads and the clip is transcribed, once with `backend: "webgpu"` and once with `"wasm"`. TE-1 passes when every request goes to `localhost:4173`, no console line starts "Refused to", and both backends return a transcript. TE-2 passes when probabilities are present at no more than 10% extra time; otherwise `confidence` stays 1.0. With probabilities on, read the confidence of the words of the dollar span: each must be 0.80 or more. Below that no event is created and V2 shows nothing: stop, and record the choice as a V2 decision in §2 (TE-2 read as failed for V2, or the clip recorded again). The threshold is not lowered. **Stop rules.** A request to any other host, or a CSP refusal: TE-1 failed; second runtime behind the same `whisper-runtime.ts` interface (TS §16 contingency). Never add a CSP host. **Human.** E-3 is not timed on R1 in this prompt: the founder's reading of 2026-10-09 stands (D-67; about 150,000 ms for the reference clip, against a line of 180,000 ms; it is in `experiments.md`). Push and read `ci`. **Done when.**

- [ ] TE-1 and TE-2 each have a recorded outcome, pass or fallback taken; the dollar span's confidence is 0.80 or more, or the decision is recorded.
- [ ] The hook is gone: `git grep -n "zz-spike" -- web/src` is empty; G M-5 passes.
- [ ] E-3: the founder's reading of D-67 is in `experiments.md`, and it is not above 180,000 ms.

## Prompt 45 — `offcut-scene`

**Objective.** A deterministic display list for any output time: Clean captions and NumberReveal (§11, D-28, D-29; G 6.1, 6.2). **Implement.**

- Fonts: the three variable fonts of §4 from their projects' official releases, under the exact file names of §4 (`Inter-Variable.ttf`, `JetBrainsMono-Variable.ttf`, `NotoEmoji-Variable.ttf`), in `crates/offcut-scene/assets/fonts/`, with `LICENSES.md` written from the licence shipped with each. Pin `parley` to agree with the `vello` Prompt 47 will use; write the `vello`, `wgpu` and `parley` versions into `v2changelog.md` now.
- The files of G 6.2 in its order: `safe_area.rs` (TS §19.1 verbatim), `easing.rs`, `anim.rs` (time computed from `t`, never accumulated), `display_list.rs` (TS §19.2 verbatim and the types of §11.4; `FontId` has four members in the given order), `styles.rs` (three explicit arms; `Bold` and `Tech` return the Clean row), `fonts.rs`, `layout.rs`, `framing.rs` (the formula of TS §19.3 with the offset read as 0.0 on one marked line), `captions.rs`, `events/mod.rs` and `events/number_reveal.rs` (all four `EventParams` arms matched explicitly; count-up runs shaped at layout time), `lib.rs` (`build_scene`, `frame_at`, `frame_count`, `summary`).
- No `HashMap`, no clock, no randomness; no timestamp is moved.

**Tests (inline).** The 11 rows of §11.8. **Validate (must fail, then revert).** `wgpu` as a dependency of `offcut-scene` (`cargo deny` and the pure-crate check). A `HashMap` in `captions.rs`: `clippy.toml` has no entry for it, so nothing fires; note the gap and check by reading. **Done when.**

- [ ] `cargo test -p offcut-scene` passes the 11 rows; clippy is clean; `check-file-tree` accepts `offcut-scene` to `offcut-text`.
- [ ] `git ls-files | grep -ci "\.\(ttf\|otf\|woff2\?\)$"` is 3, all under `crates/offcut-scene/assets/fonts/`.

## Prompt 46 — `offcut-detect`, `offcut-entitlement`, token vector

**Objective.** NumberReveal events above threshold with stable ids, and an `ExportProfile` that only a verified token can produce (§9, §10, §23.4, D-25, D-26, D-47, D-49, D-54, D-60; G 6.3–6.5). **Implement.**

- `offcut-detect`: `config.rs` (the four thresholds 0.85, 0.85, 0.80, 0.80; scoring 0.50, 0.30, 0.20, 0.10; lead 150; hold 1,400), `event_id.rs` (FNV-1a 64 over the 9 bytes of D-47), `number.rs` (score clamped to 1.0, then times the lowest anchor confidence; `label: None`), `lib.rs` (`detect`, the five steps of §9.2 with the overlap rule of D-49).
- `offcut-entitlement`: `ed25519-dalek` without default features and without `rand_core`, `base64`, `serde_json`; `claims.rs` (TS §10.6 verbatim, `deny_unknown_fields`), `token.rs` (D-25; over 2,048 characters is `Format` before any decoding), `verify.rs` (expiry is not checked here; an invalid key is skipped), `profile.rs` (the three constants, `export_profile`, the only builder of an `ExportProfile`; the table of §10.4).
- `web/tests-e2e/helpers/fake-api.ts`: generate the test seed now (`node -e` with `crypto.randomBytes(32)`) and write it as `TEST_ENTITLEMENT_SEED`; `testPublicKeyBase64`, `mintEntitlementToken`, `seedEntitlement`. Record the test public key in `v2changelog.md`; it is a public value.
- The cross-language vector: mint one token with fixed claims through a temporary `zz-mint.spec.ts` (G 6.5) and paste token, claims and public key bytes into the Rust test as constants.

**Tests (inline).** The 9 rows of §9.4 and the 9 rows of §10.5. **Validate (must fail, then revert).** One character changed in the pasted token's first segment: the vector test fails with `Signature`. **Human.** Push; read `ci`. **Done when.**

- [ ] `cargo test -p offcut-detect -p offcut-entitlement` passes 18 rows; `cargo tree -p offcut-entitlement -e normal | grep -ci "rand\|getrandom"` is 0.
- [ ] No returned event is below `threshold_number` (INV-6); the same inputs give the same ids.
- [ ] The seed occurs in `fake-api.ts` only (`git grep` over `web/src` and `crates` is empty); `sh scripts/check-gen-clean.sh` shows no diff; G M-6 passes.

## Prompt 47 — `offcut-render`

**Objective.** One wgpu pass that draws the rotated, cropped source frame and the Vello overlay to a canvas (§12, D-61; G 7.1, 7.2). **Implement.**

- Pin `vello` and the exact `wgpu` it depends on (default features off, WebGPU backend only); the `web-sys` features the crate needs. `cargo tree -d -p offcut-render` shows no duplicated `wgpu`, `peniko` or `skrifa`. Add the `cargo deny` wrappers it names and record each.
- Settle the unstable-API flag with the test of G 7.1. With the locked `web-sys` (0.3.106) the types need no flag. If the wasm32 check fails on a `wgpu` item that mentions `VideoFrame`, the flag goes on the `RUSTFLAGS` line of `scripts/build-wasm.sh`; a `.cargo/config.toml` alone never reaches the bundle.
- `gpu.rs` (one adapter, one device; device-lost sets a flag the next `render` reports), `shaders/video.wgsl` and `video_pass.rs` (axes swapped for `R90`/`R270`; `crop` in display coordinates; the frame import behind `#[cfg(target_arch = "wasm32")]`), `vello_backend.rs` (the stroke, then the fill over it, D-68; fonts by `FontId`), `composite.rs` (premultiplied alpha), `lib.rs` (`Renderer::new`, `resize`, `render`; `pub use offcut_scene as scene;`; `RenderError` with six variants). It never closes a `VideoFrame`.

**Not yet.** `render_to_image`, `golden_frames.rs` (V4). No V2 test runs this crate natively. **Done when.**

- [ ] `cargo check -p offcut-render --target wasm32-unknown-unknown` and native `cargo clippy -p offcut-render --all-targets -- -D warnings` both pass.
- [ ] The flag outcome and the three pinned versions are in `v2changelog.md`.

## Prompt 48 — `offcut-wasm-render`, bundle, loader, preload

**Objective.** `offcut_render.wasm` builds beside `offcut_core.wasm` and exports session, detection, profile and muxer (§13.4, §13.5, §15.5, §22.4, D-30, D-38, D-61; G 7.3–7.5). **Implement.**

- `Cargo.toml`: `offcut-render`, `offcut-detect`, `offcut-mp4`, `offcut-entitlement`, `offcut-types`, the binding crates of D-54; not `offcut-scene`.
- `lib.rs` (the V1 panic hook, the `{ code, detail }` helper), `detect_api.rs` (`DetectorConfig::default()`), `profile_api.rs` (non-32-byte array elements skipped; every `TokenError` is `E_ENTITLEMENT_INVALID`), `mux_api.rs` (`JsMuxSink` calls `sink.writeAt`; a throw is `IoError::Write`), `session.rs` (TS §19.2 verbatim and the four `DemuxerHandle` methods of §13.5; scene items named through `offcut_render::scene`).
- `scripts/build-wasm.sh`: `BUNDLES` gains `offcut-wasm-render:render`. `web/src/wasm/load-render.ts` (§15.5; the only instantiation site). `pool.preload()` now also calls `preloadRender()`.
- `scripts/check-hosts.mjs`: one `chunk`-scoped entry per literal inside `offcut_render_bg-*.wasm`, each with its reason (font name table, dependency error text).

**Validate (must fail, then revert).** `offcut-scene` as a dependency of this crate (`check-file-tree` and `cargo deny`). `offcut-dsp` as a dependency (`check-file-tree`). One new entry's `chunk` pointed at `offcut_core_bg-` (`check-hosts`). **Review.** Read `profile_api.rs` once for any use of the token besides `verify_token`; no tool catches a token in an error `detail`; write "checked" in `v2changelog.md`. **Browser check (dev).** Both `offcut_core_bg` and `offcut_render_bg` are fetched once at app start; `crossOriginIsolated` is `true`. **Human.** Push; read `ci`. **Done when.**

- [ ] `pnpm build:wasm` writes both bundles; the `wasm-bindgen` crate and CLI versions are equal; the render bundle's size is in `v2changelog.md`.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass; `ls web/dist/assets | grep -c "offcut_render_bg-.*\.wasm$"` is 1; G M-7 passes.

## Prompt 49 — Verifier, encoders, sink, video source

**Objective.** The independent judge of an export, written before the export, and the three building blocks of the loop (§16.7, §16.9, §23.9, D-26, D-45; G 8.1, 8.2 rows 1–3). **Implement.**

- `verify/verify_mp4.py`: checks 1 to 6 of §23.9; the invocation of TS §27.2; one line per check, a final line listing 7 to 11 as not implemented; exit 0 only when 1 to 6 pass. It runs `ffprobe` and `ffmpeg` as subprocesses and imports nothing from `crates/` or `web/`. `verify/requirements.txt` (NumPy, pinned), `verify/README.md`, the root script `verify` (`python verify/verify_mp4.py`).
- `workers/render/encoders.ts`: three names added and no V1 constant touched: `ENCODE_QUEUE_MAX = 4`, `AAC_PRIMING_SAMPLES = 0`, `pickVideoConfig(profile)` (first supported ladder entry at the profile's size and bitrate, else `E_ENCODE_VIDEO`).
- `workers/render/opfs-sink.ts` (TS §21.1; one sync handle; `abort` removes the file).
- `workers/render/video-source.ts`: the `DemuxerHandle` interface (D-30), `VideoSource` (TS §20.1; `frameAt` with a queue of 6, `frameAtBlocking` with a queue of 8 and never a stale frame), and `liveFrames`, incremented at every obtain and decremented at every `close()`.

**Validate (the verifier must fail before it is trusted).** On the reference clip with `--profile creator --expected-duration-ms 74705`: `FAIL 2`, `FAIL 3` and `FAIL 5`, exit 1. On a 2-second copy made with `-movflags -faststart`: `FAIL 1` first. **Done when.**

- [ ] Both verifier drills print the stated failures; `git grep -n "import " verify/verify_mp4.py | grep -c "crates\|web"` is 0.
- [ ] `grep -c "8_000_000\|8000000"` finds the Creator bitrate in both `encoders.ts` and `profile.rs`, with equal values; `tsc` and ESLint are clean.

## Prompt 50 — Export loop, render worker, first export

**Objective.** Every output frame rendered and encoded exactly once, streamed to OPFS as an MP4 the verifier can read (§16.6, §16.8, D-33, D-45; G 8.2 rows 4–6, 8.3). **Implement.**

- `workers/render/export-loop.ts`: `runExport`, TS §21.3 verbatim with the fixed points of §16.8 (`t = n x 1000 / 30` in integers; `vf.close()` in `finally`; the muxer created on the first video chunk; audio padded to `N x 1600` samples, never trimmed; `pts` shifted by `AAC_PRIMING_SAMPLES`; the two stage timings). The capture method is method A until Prompt 51.
- `workers/render.worker.ts`: `openSession`, `detect`, `setScene`, `attachPreview`, `exportClip` (the nine steps of §16.8), `closeSession`. `exportClip` restores the preview only when a preview canvas was attached. `previewPlay`, `previewClock`, `previewPause`, `redetectSentence`, `previewSeek` answer `E_INTERNAL` for now. Every failure carries its stage. The live-frame counter must be 0 at the end, else `E_INTERNAL`.
- The `render` row in `pool.ts`, with its one-way and during-preview lists: the table has three rows.

**Validate (must fail, then revert).** `ENTITLEMENT_PUBLIC_KEYS` imported in `media.worker.ts` (boundaries, D-27 c). **Browser check (preview, build with the test key exported in this terminal only).** The script of G 8.3 through the `zz-spike.ts` hook, with a Creator token minted by `mintEntitlementToken` (`exp` and `periodEnd` in the future): `events.length` is at least 1; no "Refused to" line; the file `exports/<exportId>.mp4` exists. Run `python verify/verify_mp4.py` on it with `--profile creator --expected-duration-ms 74705`: checks 1, 2, 3, 4 and 6 pass (2,242 frames); check 5 may fail until Prompt 51 sets the priming. With one character of the token changed: `E_ENTITLEMENT_INVALID`, and `exports/` gains no file. Remove the hook; `unset` the variable. **Done when.**

- [ ] The first export passes verifier checks 1 to 4 and 6; the wrong token is refused before anything is written.
- [ ] `git grep -n "zz-spike\|zz_spike" -- web` is empty; the gate build holds no test key.

## Prompt 51 — TE-3, TE-4, first E-4 (gate)

**Objective.** Answer the second question that can stop the project, and fix the capture method, the ladder entry and the priming (§24.1, §24.2, §24.3, D-33, D-64; G 8.3–8.5). **Implement (with the temporary hook and a keyed build, as in Prompt 50).**

- TE-3: export with method A (`new VideoFrame(canvas)`) and method B (texture readback); compare ten sampled frames; time a render-only pass over all 2,242 frames; export once with the tab hidden and verify the file. Name the chosen method in `export-loop.ts` and in TS §21.4.
- TE-4: log the entry `pickVideoConfig` returns for 1080x1920 and for 720x1280; measure the priming with the `ffprobe` line of G 8.4 and the first audio chunk's timestamp; set `AAC_PRIMING_SAMPLES` with a TE-4 comment; export again. A bitrate change goes to `encoders.ts` and `profile.rs` together, and to TS §21.2.
- `docs/v2/experiments.md`: TE-3, TE-4 and E-4. Note that the priming is the Windows encoder's and no other platform was measured (D-65).

**Validate (must fail, then revert).** Remove the `vf.close()` in `export-loop.ts` and export: `exportClip` fails with `E_INTERNAL` (D-45). **On D1, with the same temporary build; the agent takes the readings (D-69).** The TE-3 and TE-4 readings (at least 30 frames per second render-only; the ladder entry), then three exports: the `render_encode` median and its value normalised to 60 s; one file verified. At most 112,000 ms: continue. 112,000 to 187,000: continue and record the miss. Over 187,000: stop; the Canvas2D overlay path and the 60 s and 30 fps cap (§24.3). `mux` budget: 2,500 ms. Then the agent removes the hook. **Human.** Push; read `ci`. **Done when.**

- [ ] A spike export made after `AAC_PRIMING_SAMPLES` was set passes all of checks 1 to 6; the white frame near 4.99 s is white at the same second in the output.
- [ ] TE-3 and TE-4 are recorded with the method and the ladder entry; no build output contains the test key; G M-8 passes.
- [ ] E-4: the median on D1 is recorded and is not above 187,000 ms (D-69).

## Prompt 52 — Machines, stores, blockers, entitlement repo

**Objective.** The state layer of the one path, every change through a machine table (§15.3, §18, D-18, D-19, D-20, D-23, D-51, D-57; G 9.1–9.3). **Implement.**

- `state/machines/clip-machine.ts` (17 pairs) and `export-machine.ts` (16 pairs), the complete tables of TS §12.2; the absent pairs of TS §12.3 stay absent.
- `state/clip-store.ts` (the state of §18.2 with `out48`; `reset()` drops it; feed lines are appended, never dropped), `state/preview-store.ts` (its table inside the file; the seek events exist and are never fired), `state/export-store.ts` (with `unavailable`).
- `state/blockers.ts`: `BLOCKER_CODES` (eight, in the order of TS §12.5), `forImport()` complete, `forExport()` with its two state-only conditions; no side effect.
- `scripts/check-copy-codes.mjs` reads `BLOCKER_CODES`; `messages.blockers` gains four keys; the other four codes enter `COPY_PENDING`. All in this commit.
- `persistence/entitlement-repo.ts`: `get()` and `put()`; key `"current"`; `undefined` for an absent record and for a newer `schemaVersion`; it never parses the token.

**Not yet.** Machine tests (V4, V5); `blockers.test.ts` (V5). **Validate (must fail, then revert).** Delete `messages.blockers.B_PIPELINE_BUSY`; add `B_UNSUPPORTED` to `COPY_PENDING`; add a ninth code to `BLOCKER_CODES`: the copy check fails each time. **Browser check (dev).** `accepted(...)` on an `idle` clip store is refused by `transition()` and the status stays `idle`. `start(id)`, `clear()`, `encoded()`, `finalized()`, `saved()` on the export store give `gating`, `rendering`, `muxing`, `saving`, `done`. The repo: `get()` is `undefined`; after `put("a.b")` it returns `{ token: "a.b", storedAt }`; the record has key `current` and `schemaVersion: 1`. **Done when.**

- [ ] The three drills fired; `grep -o '"B_[A-Z_]*"' web/src/state/blockers.ts | sort -u | wc -l` is 8.
- [ ] ESLint is clean for `src/state`: the type-only imports of `workers/protocol.ts` are accepted (D-27 f).

## Prompt 53 — Preview loop, preview handlers, `control-preview`

**Objective.** A preview driven by the audio clock (§16.6, §16.7, §19.4, D-58, D-59; G 9.4, 9.5 row 1). **Implement.**

- `workers/render/preview-loop.ts`: `runPreview`, TS §20.2 verbatim; the loop never drives time; a late frame reuses the last one and is counted. No resize rule (V4).
- `workers/render.worker.ts`: the real `previewPlay`, `previewClock` (one-way), `previewPause`. After `previewPlay` returns, the live-frame counter must be 0. `redetectSentence` and `previewSeek` still answer `E_INTERNAL`.
- `usecases/control-preview.ts`: `attach`, `play`, `pause`, `detach`, `lockForExport`, `unlockAfterExport`; one `AudioContext`; one `TimeMs` minting helper; `previewClock` through `notify` every 250 ms; `preview_played` tracked on the first play only.

**Browser check (dev, model cached).** Take the clip to a scene by hand (import, extract, transcribe, unload, `openSession`, `detect`, `setScene`), then `attach(canvas, out48)` and `play()`: two screenshots of the canvas 1 s apart differ and neither is one flat colour. After `pause()`, two screenshots 500 ms apart are identical and no failure is stored. `play()` again resumes. **Done when.**

- [ ] The three preview results hold; `grep -c "previewPlay\|previewClock\|previewPause" web/src/workers/render.worker.ts` is at least 3.
- [ ] `tsc` and ESLint are clean for `src/workers` and `src/usecases`.

## Prompt 54 — Use-cases: pipeline, import, export, app start

**Objective.** One call takes a file to a `ready` clip, and one call takes a seeded token to a downloaded MP4 (§19, D-22, D-35, D-40, D-43, D-48, D-52, D-58; G 9.5 rows 2–5). **Implement.**

- `usecases/run-pipeline.ts`: the 12 steps of §19.3; `unload()` always runs before the render session opens (INV-12); the two stand-ins (`out48 = pcm48`, neutral prosody) each carry a `V3:` comment naming §25; stage timings measured around the worker calls (D-35); every `*_ms` capped at 3,600,000.
- `usecases/import-clip.ts`: `importClip` (9 steps), `importSampleClip`, `dismissClip`, `newClipId` (one `ClipId` minting helper). The sweep of `clips/` carries a `V7:` comment.
- `usecases/start-export.ts`: the 11 steps of §19.5; `planOf` feeds the two analytics props and nothing else; `newExportId` is a UUIDv7 (one `ExportId` minting helper); the sweep of `exports/` carries a `V7:` comment; the download name is `offcut-<yyyymmdd-hhmm>.mp4`.
- `usecases/start-app.ts`: one line, step 8: `void modelManager.inspect()`.
- The eight analytics events of §1.1 are tracked here and in `control-preview.ts`, not in components. No `client_error` for a pipeline failure (V4). No use-case calls the API (INV-3).

**Validate (must fail, then revert).** `import { startExport } from "./start-export";` in `run-pipeline.ts` (not one of the two D-58 pairs). A second `ClipId` cast in `import-clip.ts`. `fetch("/x")` in `start-export.ts`. **Browser check (dev, model cached).** `importClip(file, "user")` reaches `status: "ready"` with `events.length >= 1`, `feed[0].kind === "transcribing"` and `out48.length === 3,585,840`; during it the only requests are `POST /api/v1/events`. With no `entitlement` record, `startExport(clipId)` sets `unavailable: true`, the status stays `idle`, and `exports/` is empty. With the dev server started with the test key and a seeded Creator token, `startExport` ends `done` and the downloaded file passes verifier checks 1 to 6. `dismissClip()` returns the store to `idle` and leaves `clips/` empty. **Human.** Push; read `ci`. **Done when.**

- [ ] The four browser results hold; `git grep -n "V3: \|V7: " -- web/src/usecases | wc -l` is 4.
- [ ] `git grep -n "file\.name\|webkitRelativePath" -- web/src` is empty; the three drills fired; G M-9 passes.

## Prompt 55 — Copy, components, pages, landing navigation

**Objective.** A person can drop the clip, watch the feed, play the preview and, with a seeded token, download the MP4 (§20, §23.5, D-48, D-56; G 10.1–10.5). **REOPENED V1 CONTRACT:** `web/tests-e2e/landing.spec.ts`. **Implement.**

- `copy/messages.ts`: `feed`, `preview`, `export`, `editor` (§20.3). `export.unavailable` says exporting needs an account and accounts are not open yet; no date. New classes in `ui/styles/pages.module.css` and `components.module.css`, with the tokens of `tokens.css`; no `style` attribute string (§20.4).
- `ProcessingFeed.tsx` (a `switch` over `kind` and over `eventKind`, each ending in `assertNever`; no invented line), `PreviewPlayer.tsx` (canvas 540x960; `attach` on mount, `detach` on unmount; `pause()` when the tab hides; buttons disabled while `locked`), `ExportButton.tsx` (reads `forExport()`; the `unavailable` copy; no counter, no upgrade prompt), `ExportProgress.tsx` (stage, bar, elapsed, the "rendering on your computer" line).
- In one step, because each makes the others true: `DropZone.tsx` wired to `importClip(file, "user")` and `importSampleClip()`, the first dropped file only; `messages.dropZone.notReady` deleted; `LandingPage.tsx` navigates to `appPath` when the clip leaves `idle`; `routes.tsx` passes `appPath` (add it to the "CHANGED" list of §4); `EditorPage.tsx` renders by clip status (the table of §20.1), its placeholder and `NotifyMeForm` removed; `landing.spec.ts`: the two cases of §23.5 replace "the drop zone is inactive" and "/app shows the not-ready panel", every other case untouched.

**Validate (must fail, then revert).** `<p>Rendering</p>` in `ExportProgress.tsx` (`react/jsx-no-literals`). `workers/pool` imported in `PreviewPlayer.tsx` (boundaries). `useClipStore.setState(...)` in a component: V1 has no rule for it; note the gap. **Browser check.** In dev, the rows of G 10.4 an agent can read: `/` to `/app` on a drop; "Transcribing…" first; at least one "Found:" line with `$12k`; no "Cleaning voice" line; the player; replay at the end; the unavailable message with no token; the stub and "start over" for a text file renamed `.mp4`; the model panel after `models/` is deleted; the sample button. In a keyed production build with a seeded token, the rows of G 10.5: the stage labels in order, the download name, a working preview afterwards, no "Refused to" line; the file passes verifier checks 1 to 6. **Human.** Watch the preview and the exported file once: a 9:16 centre crop, captions in the lower third word by word, sound in sync, the reveal as the amount is spoken. Push; read `ci`. **Done when.**

- [ ] `git grep -n "notReady" -- web` is empty; `pnpm e2e` passes 12 cases with the two replaced ones.
- [ ] Both walk-throughs match every row; the exported file passes checks 1 to 6; G M-10 passes.
- [ ] No banned phrase in the copy: `git grep -n "never leaves\|GDPR\|DPDP\|CCPA\|SOC 2\|compliant" -- web/src/copy` is empty.

## Prompt 56 — E2E helpers, setup project, `model-download.spec.ts`

**Objective.** The media suites can run from one command without touching `pnpm e2e` (§22.2, §22.3, §23.4, §23.6, D-39, D-41, D-63; G 11.1, 11.2 row 1). **New file in the tree:** `web/tests-e2e/media.setup.ts` (add it to TS §5 and §4 in this commit). **Implement.**

- `helpers/fixtures.ts`: `routeAssets` (answers asset-host URLs from `fixtures/.cache/` and `referenceClip()`, honours `Range`, records; with `E2E_REAL_ASSETS=1` it only records), `dropClip` (names the file `clip.mp4`), `ensureModelCached`, `opfsList`, `sourceDurationMs`, `referenceClip`, and `fillAssetCache()` (downloads what the cache lacks from the asset host and checks each SHA-256).
- `media.setup.ts`: one Playwright setup test that calls `fillAssetCache()`, its timeout raised for a download of up to 260 MB. The config has no `globalSetup`.
- `web/playwright.config.ts`: projects `media-setup`, `media` (the three suites, `channel: "chrome"`, 300 s, `dependencies: ["media-setup"]`) and `bench` (`testDir: "../bench"`, headed, the same dependency); `non-media` gets `testMatch` of `landing.spec.ts` and no dependency.
- Root `package.json`: `e2e:media`, `e2e:device`, `bench:device`.
- `model-download.spec.ts`: the 8 cases of §23.6. Every media suite calls `installFakeApi`.
- `fixtures/speech/README.md`: the expected transcript and expected events, final.

**Validate.** `pnpm e2e` on a machine whose `fixtures/.cache/` is empty requests nothing from the asset host for the model (the setup project does not run). In `routeAssets`, answering one model request for a host that is not the asset host makes the "Requests" case fail; revert. **Run.** `export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=<test public key>; pnpm build && pnpm e2e:media; unset VITE_ENTITLEMENT_TEST_PUBLIC_KEY`, then the gate on a plain build. **Done when.**

- [ ] `pnpm e2e:media` passes 8 cases; `pnpm e2e` passes 12 and downloads no model.
- [ ] `pnpm --filter web exec tsc --noEmit -p tests-e2e/tsconfig.json` is clean.

## Prompt 57 — `pipeline-preview.spec.ts`, `export-creator.spec.ts`

**Objective.** The one path and its export, asserted in the browser, with the verifier called from inside the suite (§23.7, §23.8; G 11.2 rows 2–3). **Implement.**

- `pipeline-preview.spec.ts`: the 11 cases of §23.7 (`clip_accepted` with `source: "user"`, `orientation: "landscape"`, `duration_bucket: "lte90"`; no `audio_chain` timing; exactly one `preview_played` after two plays; the long-task case writes its number and asserts nothing).
- `export-creator.spec.ts`: the 11 cases of §23.8; each export is checked by `verify_mp4.py`; the free-plan token gives 720x1280; a token signed with another seed gives `E_ENTITLEMENT_INVALID` and an empty `exports/`.

**Validate.** Built without the test key, the "Creator export" case fails with `E_ENTITLEMENT_INVALID`: the suite depends on the key, as intended. Commenting out `await pool.asr.unload()` in `run-pipeline.ts`: nothing in V2 asserts INV-12; note it as covered by review until V4, and restore the line. **Run.** As in Prompt 56, with the keyed build for the media suites and a plain build for the gate. **Done when.**

- [ ] `pnpm e2e:media` passes 30 cases (8, 11, 11); no feed text appears in any request.
- [ ] After the plain build, `grep -rl "<test public key>" web/dist | wc -l` is 0.

## Prompt 58 — Bench, media job in CI, test-key guard

**Objective.** The media suites and the verifier run in CI on Windows before any deploy, a test key can never reach production, and the cost of the job is known (§22.5, §23.10, D-24, D-44; G 11.3–11.5; TE-10). **Implement.**

- `bench/device-bench.ts` (§23.10): one warm-up, ten runs; the result file of §23.10, numbers and enums only.
- `.github/workflows/e2e-media.yml`: reusable, `windows-latest`, `defaults.run.shell: bash`; Node, pnpm, Playwright's Chrome, Python through `actions/setup-python` pinned to a commit, `ffmpeg` from an archive with its SHA-256 written in the file; the `web/dist` artifact; the cache of `fixtures/.cache/` keyed by the hash of the manifest; `pnpm e2e:media`; on failure the HTML report only, never an MP4 or a trace.
- `.github/workflows/ci.yml`: step 8 builds with the test public key written literally and uploads `web/dist` on every event; step 10 calls the media workflow before any deploy; the Vercel step is split into build, the test-key guard, deploy. The guard searches `.vercel/output/`. The `wasm-pkg` artifact carries both bundles.
- `scripts/check-external-facts.mjs`: the TE-10 line with its date. `docs/v2/experiments.md`: TE-10.

**Validate.** `BENCH_DEVICE=dev pnpm bench:device` writes a file with 10 runs and the five stages; `grep -c "clip.mp4\|twelve"` in it is 0; delete it. The guard's search finds the key in a keyed build (1 or more) and not in a plain one (0). Both workflows pass `actionlint` if it is installed. **Human.** Push; mark the pull request ready; read every job. WebGPU missing on the runner: add the launch arguments and record them. `UNSUPPORTED_AAC_ENCODE` or `UNSUPPORTED_H264_ENCODE` on the runner, or minutes that do not fit the allowance: the TE-10 fallback (step 10 on `main` and manual trigger only; `pnpm e2e:device` before every merge). If the secret scan names the test public key line, the agent ends it with `# gitleaks:allow`. **Done when.**

- [ ] The local chain is green with 30 media cases; no `dev-*.json` is left in `bench/results/`.
- [ ] The pull request is green including the Windows job, or the fallback is in the workflow and recorded; TE-10 has minutes and a date; G M-11 passes (Human).

## Prompt 59 — Deploy and measure

**Objective.** V2 live and checked from outside, with R1 numbers committed (§24.1, §26 deployment and experiment groups; G 12.1–12.5). **Implement (agent).**

- The final-values table of G 12.1, one row at a time, with evidence: the manifest, `SAMPLE_CLIP_PATH`, the production public key, `AAC_PRIMING_SAMPLES` and the ladder, equal bitrates, the runtime options, the capture method, the test key literal and the guard.
- After the human's merge: the commands of G 12.3 (`healthz` reports the merge commit; the three headers; `/ort/<version>/` served as `application/wasm` and immutable; zero hits for the test key in the deployed scripts).
- The TE-14 clip with the `ffmpeg` line of G 12.5 (`-t 89.9`; it stays in `testclips/`).
- `docs/v2/experiments.md`: E-3, E-4, TE-14 and the cold capability-check time from the numbers the human hands over, each also normalised to 60 s. Commit `bench/results/d1-<date>.json`.

**Human.** `vercel env ls production` shows no `VITE_ENTITLEMENT_TEST_PUBLIC_KEY`. Merge the pull request; watch `ci` on `main` (media job, API, wait for the version, build, guard, deploy, header check, `@smoke`). On R1, on the deployed page: the rows of G 12.3 (model from the asset host, feed, preview; export unavailable; the sample button). The analytics rows through `PROD_DATABASE_URL`, never `$DATABASE_URL`. On R1 with the keyed build: `E2E_REAL_ASSETS=1` first-run timing, `pnpm e2e:device`, `BENCH_DEVICE=d1 pnpm bench:device`, one export verified, and TE-14 (ten runs; peak under 1.5 GB; no tab crash). **If a step fails.** `wait-for-version` times out: fix the API, never deploy the web first. The guard fails: remove the variable from Vercel. The header check or `@smoke` fails after the deploy: `vercel rollback`, then fix forward. **Done when.**

- [ ] `ci` is green on `main`; the deployed bundle holds no test key; headers, `@smoke` and isolation hold.
- [ ] The deployed page on R1 downloads the model, shows the feed and plays the preview; five event names appear in `analytics_events` with enum and integer props only (Human).
- [ ] `bench/results/d1-<date>.json` is committed with 10 runs; E-3, E-4 and TE-14 are recorded; `pnpm e2e:device` passed on D1 (D-69).

## Prompt 60 — Records, spec corrections, M0 gate, tag `v2`

**Objective.** A verified, recorded, tagged V2, and the decision whether V3 starts (§24.3, §25.3, §26, §27; BP §4.3, §4.4; G 12.6, 12.7). **Implement.**

- `docs/technicalspec.md`: every assumption V2 measured replaced by its value in §39.3; the corrections of §27 items 1, 2, 4, 7, 12 and 14; the decisions listed at the end of §27; D-65.
- `docs/buildplan.md`: the schedule moves D-18, D-20, D-21, D-23, D-24, D-33; D-65 and D-69 in §1.5, §4.2 to §4.4 and §12.1. `docs/product.md`: D-65 and D-69 in A-7 and §20.1.
- `docs/v2/v2implementation.md`: any D-n that changed during the build; in §4, `routes.tsx`, `web/tests-e2e/media.setup.ts` and `.cargo/config.toml` if Prompt 47 created it; the decisions the guide added (the short-file rule; the confidence decision, if one was taken).
- The five audits of G 12.6: no `fetch(` outside the four files; no font outside the fonts directory; no source file over 400 lines; no TODO without a version; an empty frozen-file diff against the baseline.
- Tick the 13 boxes of §25.3 and the 26 of §26, each against its evidence. Tick "Tagged `v2`" in this last commit.

**Human.** Read E-1 from the production database (G 12.7, `PROD_DATABASE_URL`). Write the M0 decision into `experiments.md`: the three numbers (E-1; E-3 and E-4 on R1, measured and normalised), the column each fell in, the action, the date. E-1 with fewer than 1,000 visitors or no announcement is "inconclusive": decide on E-3 and E-4. Continue, or a fallback applied and re-measured: `git tag -a v2`, push the tag. Stop: no tag; V3 does not start. **Done when.**

- [ ] `grep -c "\- \[ \]" docs/v2/v2implementation.md` is 0; the five audits print nothing.
- [ ] `pnpm check && pnpm test && pnpm build && pnpm e2e` is green on the commit to be tagged, and `ci` is green on `main`.
- [ ] The M0 decision is written with its date, and `git ls-remote --tags origin v2` prints one line, or the stop is recorded (Human).

---

# Cross-Prompt Contracts

| Contract | Owner → consumer | Shape | Change rule |
| --- | --- | --- | --- |
| V1 frozen contracts | V1 → everything | Types, protocol, schemas, headers, allowlist | Not edited in V2, except the three reopened files below |
| Reopened V1 contracts | P32, P43, P55 → all later prompts | `eslint.config.js` (15 entries, one runtime-import rule), `check-hosts.mjs` (`chunk`), `landing.spec.ts` (two cases) | Only what D-27, D-38, D-56, D-58, D-59 name |
| Crate graph | P32 → P34–P48 | TS §7 plus scene → text (D-28); wasm-render reaches scene through render (D-61) | `check-file-tree` and `deny.toml` together |
| `ProbeInfo`, `ClipInfo`, 11 rules | P35 → P36, P37, V3 | §6.6, §6.7 | V3 adds fixtures and copy, not rules |
| Exact PCM length | P35, P37 → P50, V3 | `pcm48.len() == round(duration_ms x 48)` (D-34) | Frozen: the voice chain returns the same length |
| `CoreApi`, `RenderApi` | P36, P42, P48 → workers | §13.3, §15.5 | Additive only |
| RPC surface, pool table | P37 → P43, P50, V4 | §15.6, §15.7 | V3 adds one row; V4 adds restart |
| OPFS path layout | P37 → workers, models, E2E | TS §23.1 | Frozen |
| Manifest shape, part size, retries | P38, P39 → P40, P56 | TS §16.1, §17.2 | A new model is a new `modelId` |
| Muxer layout and API | P41 → P48, P50 | §6.8, TS §21.1 | V5 adds `ctts` |
| `RawWord`, `Token`, `Transcript.model_version` | P42, P43 → P46, V3 | §8, D-50 | Frozen |
| Display list, `FontId` order | P45 → P47, V4 snapshots | §11.4 | Frozen |
| Event-id bytes | P46 → V3 storage | D-47 | Frozen |
| Token wire format, profile table | P46 → P48, P50, V6 `sign.rs` | D-25, D-26 | Frozen |
| Bitrates | P46, P49 → P51 | `profile.rs` equals `encoders.ts` | Both or neither |
| Machine tables, `BLOCKER_CODES` order | P39, P52 → use-cases, UI | TS §12.2, §12.5 | Frozen |
| Verifier command line and output | P49 → P57, P58, V5 | TS §27.2 | V5 adds checks 7 to 11 |
| Bench result shape | P58 → P59, V9 | §23.10 | Frozen |

# High-Risk Implementation Areas

| Risk | Why | Prompt | Validation |
| --- | --- | --- | --- |
| Worker APIs under the production CSP | Works in dev, dies deployed | 33, 44, 50 | Probe and both spikes on `vite preview` |
| ASR runtime fetching a hub or CDN | Breaks the closed network list | 43, 44 | TE-1 request capture, both backends; never a new CSP host |
| The one visible event needs confidence 0.80 | V2 shows nothing below it | 44 | Confidence of the dollar span read; decision recorded |
| `wasm-bindgen` pin, `wgpu`/`vello` pair | Two trees or a broken bundle | 36, 47, 48 | Version pair check; `cargo tree -d` |
| Unstable-API flag in the wrong place | Bundle fails while `cargo check` passes | 47 | Test of G 7.1; flag on the `RUSTFLAGS` line of `build-wasm.sh` |
| Frame leaks | Memory and a silent wrong export | 49–51, 53 | Live-frame counter in every build (D-45); the `vf.close()` drill |
| AAC priming | Check 5 fails; audio late | 51 | Measured value; re-export passes checks 1 to 6 |
| Demuxer misreading shared with its own tests | Hand-built fixtures agree with the bug | 35, 41 | Real-clip probe; `ffprobe` on a muxed file |
| Model list unknown before the runtime runs | A missing file at first load | 38, 43 | Note in Prompt 38; repeat steps 2 and 3 |
| Test key reaching production | Anyone could mint Creator tokens | 46, 56–59 | Gate checks the environment; CI guard; deployed-bundle search |
| Local database read as production | Wrong E-1 and analytics readings | 59, 60 | `PROD_DATABASE_URL`; `select current_database()` first |
| CI seen only at the end | Ten phases of surprises on day 9 | 33 onward | Draft pull request; push at every milestone |
| Windows runner without WebGPU or encoders | Media job cannot run | 58 | TE-10 fallback |
| R1 not available | Neither gate can be read on it | 31, 44, 51, 59 | It happened. Since D-69 the gates are read on D1, and a machine like R1 is measured after the deploy |

# Architectural Invariants

1. The recording's timeline is never changed: no timestamp is moved, no sample or frame is dropped because of what the audio contains (INV-5, INV-10). The export has `ceil(duration_ms x 30 / 1000)` frames.
2. No event exists below its kind's threshold (INV-6). Equal scene inputs give equal display lists (INV-7).
3. Output size and watermark come from the verified token only, in `export_profile` (INV-9).
4. Every obtained `VideoFrame` and `AudioData` is closed by the code that obtained it, in `finally` (INV-11).
5. The ASR session and the render session never coexist (INV-12).
6. A model file is used only after its SHA-256 matched the bundled manifest (INV-20).
7. The browser contacts the app origin and the asset host, nothing else (INV-1). No request carries media, transcript, caption text, event parameters or a file name (INV-2). Import, processing and preview never call the API (INV-3).
8. The main thread decodes, infers, renders and encodes nothing (INV-17); hashing in short slices is the one WASM call it makes.
9. Layer imports follow TS §2 plus the edges of D-27 and D-58. Pure crates read no clock, no randomness and no browser API.
10. V1 and V2 files are not restructured later: V3 and V4 fill bodies and add files and rows (§25).

# Forbidden Implementation Shortcuts

- No silence detection, pause handling or time map, not even as a stand-in (TS §36). No Canvas2D backend unless TE-3 or E-4 failed.
- No new CSP host, no remote model loading, no runtime file from a CDN. No `fetch` in a worker except through `wasm/load-*.ts`.
- No test file beyond the list of the SAB. No committed `zz-` file. No committed reference clip (it stays in `testclips/`).
- No test key or seed outside `web/tests-e2e/helpers/fake-api.ts` and the literal public key in `ci.yml` step 8. No test key in a `.env*` file, in Vercel or in a gate build.
- No second copy of a value that has one home: limits in `limits.rs`, bitrates in `profile.rs` (mirrored once in `encoders.ts` for the capability probe), safe-area numbers in `safe_area.rs`, thresholds in `config.rs`, OPFS paths in `opfs.ts`, copy in `messages.ts`.
- No guess where the plan is silent: stop and list. No edit to a frozen file to make something pass.
- No threshold lowered to make the reference clip show an event. No expectation adjusted to a wrong demuxer value.
- No push to `main`, no merge and no tag by the agent. No production database query by the agent.
- No widening of a `check-hosts` entry to all chunks. No loosening of an existing lint rule.

# Requirement → Prompt Traceability

| Requirement (BP §4.4, §26) | Prompt(s) | Validation |
| --- | --- | --- |
| Minimal ingest: OPFS copy, demux, probe, validate, PCM (BP §4.1 A) | 34, 35, 36, 37 | Inline rows; real clip; browser numbers |
| Model manifest, resumable hashed download (BP §4.1 B) | 38, 39, 40 | `download.test.ts`; reload with zero requests |
| ASR from OPFS only, word timestamps (BP §4.1 B) | 42, 43, 44 | TE-1, TE-2; transcript in the README |
| Muxer, `mux_roundtrip.rs` (BP §4.1 C) | 41 | 12 cases; `ffprobe` |
| Scene, renderer, render bundle (BP §4.1 C) | 45, 47, 48 | 11 rows; both targets compile; `check-hosts` |
| Export loop, encoders, sink (BP §4.1 C) | 49, 50, 51 | Verifier checks 1 to 6 |
| `verify_mp4.py` checks 1–6 (BP §4.4) | 49 | Two fail drills |
| Text, NumberReveal detection, entitlement profile (BP §4.1 D) | 42, 46 | 11 + 9 + 9 rows; cross-language vector |
| Use-cases, stores, preview, UI (BP §4.1 D) | 52, 53, 54, 55 | Browser checks; walk-throughs |
| `model-download.spec.ts`, `pipeline-preview.spec.ts`, `export-creator.spec.ts` (BP §4.4) | 56, 57 | 30 cases |
| `e2e-media.yml` runs them in CI (BP §4.4) | 58 | Green job, or the TE-10 fallback |
| ASR and render timings on R1 committed (BP §4.4, D-65) | 44, 51, 59 | `bench/results/d1-<date>.json` |
| TE-1, TE-2, TE-3, TE-4, TE-10, TE-14 recorded (BP §4.4) | 44, 51, 58, 59 | `experiments.md` |
| Eight analytics events (BP Appendix C) | 53, 54 | `pipeline-preview` and `export-creator` event cases |
| Deployed, server first, header check, `@smoke` (BP §1.2) | 59 | `ci` on `main`; G 12.3 |
| M0 gate decision written; tagged `v2` (BP §4.3, §4.4) | 60 | `experiments.md`; `git ls-remote` |
| Hand-over to V3 (§25.3) | 60 | 13 boxes with evidence |

# Final Coverage Audit

- [x] Every row of §1.1 (ingest, model, ASR, text and detection, scene and render, preview, export, verifier, workers, CI, analytics, experiments) maps to a prompt.
- [x] Every file of the §4 tree is named in a prompt, with the three the guide adds: `routes.tsx` (55), `web/tests-e2e/media.setup.ts` (56), `.cargo/config.toml` (47, only if needed).
- [x] Every decision is implemented: D-18, D-19, D-20, D-23 (52); D-21 (35); D-22, D-35, D-40, D-43, D-52 (54); D-24 (33, 58); D-25, D-47, D-49 (46); D-26 (46, 49); D-27, D-28, D-58, D-59, D-63 (32); D-29 (45); D-30 (48, 49); D-31, D-42 (42, 43); D-32 (37); D-33 (41, 50, 51); D-34 (35, 37); D-36 (36, 39, 40); D-37, D-38, D-50 (43; D-38 also 48); D-39 (33, 38, 56); D-41 (31, 56); D-44 (58); D-45 (49, 50, 51); D-46, D-65 (31); D-48, D-56 (54, 55); D-51 (52, 55); D-53, D-54 (36, 46); D-55 (41); D-57 (40, 52); D-60 (34, 46); D-61 (47, 48); D-62 (38, 39); D-64 (33, 44, 51, 59).
- [x] Every experiment has a prompt: TE-7 re-check (38); TE-1, TE-2, E-3 (44); TE-3, TE-4, E-4 (51); TE-10 (58); TE-14 and the final E-3 and E-4 (59); E-1 (60).
- [x] Tests are incremental: inline Rust rows in 34–36, 42, 45, 46; `mux_roundtrip.rs` in 41; `download.test.ts` in 39; the three media suites in 56 and 57. Exactly the test files of §23.1 and the setup file.
- [x] Every prompt ends with the same gate, so a regression is caught in the prompt that caused it; the test counts may only rise.
- [x] Dependencies are valid: every prompt needs only earlier prompts, the specs and the listed Human steps.
- [x] Exactly 30 prompts; Prompt 60 yields a deployed, measured and tagged V2 with the M0 decision written, or a recorded stop.

**Issues in the source documents that these prompts resolve or carry.** (1) `docs/v2/coding-promptsv2.md` and `v2implementation-notes.md` are missing from D-46 and the TS §5 tree: Prompt 31. (2) D-59 says "a per-file override, for one minting line each" and the guide wants a second cast to fail; a plain per-file override cannot do both: Prompt 32 exempts one named helper. (3) V1 has no lint rule for TDR-3: Prompt 43 adds it. (4) The guide makes `pnpm build` fail between its Steps 5.2 and 5.5; here the host entries are written in Prompt 43, so every commit passes the gate, and TE-1 confirms them in Prompt 44. (5) The guide generates the test key pair in its Step 0.5 and carries it in a scratch note; here Prompt 46 generates it where it is first written down. (6) §16.8 does not say what `exportClip` does when no preview was attached: Prompt 50. (7) The model's file list is only certain once the runtime runs: note in Prompt 38.

# Revision notes

Changes made to the first draft of this file (27 prompts, 31 to 57) after reading it against the two V2 documents, the three specs, the V1 plan and the repository:

| # | Where | Change | Why |
| --- | --- | --- | --- |
| 1 | Whole file | 30 prompts, 31 to 60 | V1 used exactly 30; the draft's own header assumed 30 per build and held 27 |
| 2 | Header | "I don't have the V1 prompt list" removed; the TS §34.1 deviation recorded for V2 | `docs/v1/coding-prompts.md` ends at Prompt 30 and accepted the format for V1 only |
| 3 | Preamble | Replaced by an SAB with the terminal block, the precedence rule, the frozen list, the test-file list and one gate | The draft said "the plan wins" over the specs (BP says the reverse outside §2 decisions), named no shell, no Python, no environment, and ran only `pnpm check` after a prompt |
| 4 | Every prompt | "Accept" bullets replaced by "Done when" boxes with the command or the number that proves each | A box such as "the dev-console checks hold" cannot be ticked from evidence |
| 5 | Every prompt | The same gate: four commands, the frozen-file diff, no `zz-` file, no test key in the environment, test counts that may only rise | The draft had no regression check between prompts |
| 6 | Prompt 31 | Preflight added: branch, pending documents committed, baseline recorded, the precondition checks an agent can run | The draft called Phase 0 "manual", while its first prompt needed a clean tree and five documents were uncommitted |
| 7 | Prompts 33, 36–38, 40, 43, 44, 48, 50, 52–55 | "Browser check": the agent runs the guide's console snippets through a temporary Playwright spec | The draft marked three whole prompts and several boxes as Human; only R1, dashboards, credentials, pushes and judgements by eye need one |
| 8 | Prompts 39–40, 49–50, 53–54, 56–57 | Four heavy prompts split in two | The draft's Prompts 40, 49, 52 and 54 each held several subsystems and one check at the end |
| 9 | Prompts 31, 37, 58 | Three regroupings | The preflight shares a prompt with the skeletons (as V1's Prompt 01 did); the draft's Prompts 37 and 38 are one prompt, the plan's step S4, with one real check; the bench moved from the E2E prompt to the CI prompt |
| 10 | Prompt 49 → 51 | The `vf.close()` drill moved to the spike prompt | It needs a running export, which the draft's Prompt 49 did not have |
| 11 | Prompt 50 | A first export on the development machine, verifier checks 1 to 4 and 6 | The export loop and the render worker would otherwise be committed unexecuted |
| 12 | Prompt 43 | The TDR-3 lint rule is added; the host entries are written here | V1's ESLint config has no such rule, so the draft's "restricted-import test fires" could not; and no commit should fail `pnpm build` |
| 13 | Prompt 32 | The cast overrides exempt one named helper each | A plain override would let a second cast through, against G 3.4 and G 9.5 |
| 14 | Prompt 38 | The manifest is written by a one-off command; the order (script, upload, manifest, fetch) fixed; the CORS origin moved to "before Prompt 31" | A generator script would be a file outside the tree; the origin is needed by Prompt 44 |
| 15 | Prompt 46 | The test key pair is generated here | The draft never said when; Prompt 33 needs only any 32-byte value |
| 16 | Prompt 31 | TS §5 additions name all six `docs/v2/` files | The draft omitted the guide, the notes file and this file |
| 17 | "Things I'd flag" | Removed; item 3 was wrong | It tied a model change to the scene prompt; the E-3 stop rule belongs to Prompt 44 and repeats Prompts 38 to 40 |
| 18 | End of file | Dependency table with guide steps, contracts with owners and change rules, risks with validations, invariants, forbidden shortcuts, traceability to BP §4.4 and §26, coverage audit | The draft had a five-line contract list and no audit |
