# Notes for `docs/v2/v2implementation.md`

These two parts were requested around the document, not inside it.

## Analysis (written before the document)

1. **Scope as read from `buildplan.md` §4.** V2 = M0.2 + M0.3 + M0.4: minimal ingest, model download, ASR spike, render/encode spike, then one joined path (reference clip in, 1080x1920 MP4 with Clean captions and one NumberReveal out), plus the M0 gate on Sun 1 Nov 2026.
2. **Explicitly out:** rejections UI, voice chain, three event kinds, two styles, crop offset, review controls, accounts (BP §4 "Not in V2").
3. **V1 hand-over honored:** every "V2 adds" row of V1 §15.2 and all nine items of V1 §15.3 (they became preconditions 1-9).
4. **V1 freezes kept:** types, protocol, both schemas, API tables, headers, allowlist, HTTP shapes, `transition()`, copy location.
5. **V1 freezes reopened, each recorded with its cost:** `eslint.config.js` boundaries (D-27, D-58, D-59), `check-hosts.mjs` (D-38), `landing.spec.ts` (D-56).
6. **Contradiction: use-case imports.** TS §7 forbids use-cases importing each other; TS C-2 and C-9 require it.
7. **Contradiction: boundaries.** TS §2/§7 lack four edges the spec's own flows need (sample fetch, OPFS paths in workers, public key in the render worker, model manager writing its store).
8. **Contradiction: timings.** TS §32 says stage timings come from workers; the protocol returns them only from `exportClip`.
9. **Contradiction: brand casts.** TS §10.1 limits casts to `gen/` and `workers/`; main-thread signatures produce `Bytes`, `TimeMs`, `ExportId`.
10. **Contradiction: frame counter.** A dev-build assertion (TS §21.4) is read by E2E suites that run production builds (V1 D-15).
11. **Contradiction: worker count.** V1 says V2 preloads four worker scripts; `audio.worker.ts` is a V3 file.
12. **Contradiction: hashing.** C-3 hashes while downloading; §16.3 hashes the finished part. Only the second survives a resume.
13. **Gap: undefined names.** `Token`, `DemuxerHandle`, `JsRandomAccess`, the display-list helper types, `FontId`, the Rust `RawWord`.
14. **Gap: missing crate edge.** `offcut-scene` needs `format_quantity` from `offcut-text`.
15. **Gap: token.** The signed bytes of the entitlement token are unspecified; no route can deliver a token before V6.
16. **Gap: schedule.** `DropZone` reads blockers in V2 and stores must use `transition()`, but `blockers.ts` and two machine files are scheduled later; `probe_and_validate` is V2 while `validate.rs` is V3; verifier check 5 is a V2 exit criterion while the audio edit list is V5.
17. **Gap: detection reality.** With V2's number subset, only a `$` amount of 1,000 or more clears the 0.80 threshold reliably, so the reference script must contain one.
18. **Expected decisions:** D-18 to D-60 (43), of which 6 are schedule moves and 5 reopen a V1 contract. (The revision of 2026-10-08, at the end of this file, added D-61 to D-63.)

## Consistency check (written after the document)

### 1. Every V1 hand-over entry for V2 (V1 §15.2) and where it is satisfied

| V1 file | V1 said V2 adds | Section |
|---|---|---|
| `Cargo.toml` | Eight members | 22.1 |
| `offcut-types/*`, `offcut-api-types/*` | Nothing | 14, 21 |
| `offcut-wasm-core` | `media_api.rs`, `hash_api.rs`, `text_api.rs`; `CoreApi` grows | 13.1-13.3 |
| `scripts/build-wasm.sh` | One `BUNDLES` entry | 22.4 |
| `server/*` | Nothing | 14 |
| `workers/protocol.ts` | Nothing; `rpc.ts`, `pool.ts` and the workers implement it | 15.6, 15.7, 16 |
| `workers/pool.ts` | Lazy creation, typed clients, preload of worker scripts and the render bundle | 15.7 (three scripts, D-22) |
| `workers/render/encoders.ts` | `pickVideoConfig()` | 16.9 |
| `wasm/load-core.ts` | Used by the media and ASR workers; `load-render.ts` beside it | 13.3, 15.5 |
| `persistence/schema.ts` | Nothing | 4 (untouched list) |
| `persistence/db.ts` | `opfs.ts` appears beside it | 15.2 |
| `net/asset-fetch.ts` | `fetchAsset` with HTTP Range | 15.4 |
| `net/http.ts`, `api-client.ts` | Nothing | 4, D-23 |
| `analytics/client.ts` | Nothing; eight more events tracked by use-cases | 19.2-19.5 |
| `platform/capability.ts` | Nothing | 4 |
| `state/*` | Four stores and the model machine, all using `transition()` | 18 (plus D-18, D-19) |
| `usecases/start-app.ts` | `modelManager.inspect()` at the marked position | 19.1 |
| `copy/messages.ts` | Model download, feed, preview, export copy | 20.3 |
| `ui/components/DropZone.tsx` | Wired to `importClip`; reads clip store and blockers | 20.2, 18.5 |
| `ui/pages/EditorPage.tsx` | Real editor shell | 20.1 |
| `ui/pages/LandingPage.tsx` | Sample-clip button active | 20.1, 20.2, 19.2 |
| `web/vite.config.ts` | ORT `.wasm` copy | 22.3 (D-37) |
| `web/vercel.json` | Nothing | 22.6 |
| `ci.yml` | `e2e-media.yml` beside it | 22.5 |
| `scripts/check-copy-codes.mjs` | Pending list shrinks | 22.4 (D-57) |
| `scripts/upload-assets.sh` | Uploads the model files | 22.4, 22.6 |
| `tests-e2e/helpers/fake-api.ts` | Creator token signed with a test key | 23.4 (D-23, D-24) |
| `fixtures/speech/README.md` | Reference clip recorded | S1, D-39 |

V1 §15.3 items 1-9 are preconditions 1-9 of section 1A, in the same order.

### 2. TS invariants and limits V2 touches, and where they are enforced

| Invariant or limit | Enforced in |
|---|---|
| INV-1 closed host list | 15.4 (`fetchAsset`), 16.5 (cache adapter never fetches), 22.4 (`check-hosts`, D-38), 23.6, TE-1 |
| INV-2 no content in requests | 19.3 (feed text stays local), 23.7 "Event shape" |
| INV-3 no API during import, processing, preview | 19.2-19.4; 23.7 "Quiet processing" |
| INV-5 timeline preservation | 6.5 (time base), 11 (no timestamp moved), 16.3, 16.8, verifier checks 3 and 5 |
| INV-6 no event below threshold | 9.2 step 3; 9.4 |
| INV-7 deterministic display lists | 11 crate rules; 11.8 |
| INV-8 exact frame count at 30 fps | 6.8, 16.8; verifier check 3 |
| INV-9 size and watermark from the verified token only | 10.3, 10.4, 16.6, 16.8; 23.8 |
| INV-10 audio duration preservation | 7 (length rule), 16.1 (D-34), 16.8 (pad only) |
| INV-11 every frame closed | 16.7 (`liveFrames`), 16.8, D-45 |
| INV-12 ASR and render sessions never coexist | 16.4 `unload`, 19.3 step 7 |
| INV-15 copy and numbers from `messages.ts` and limits | 20.2, 20.3 |
| INV-16 every state change through `transition()` | 18 |
| INV-17 main thread does no media work | 15 layer rules, 16; D-36 keeps hashing in short slices |
| INV-18 analytics without free text | 19.2-19.5 props; 23.7 |
| INV-20 model used only after hash match | 16.5, 17.3; 23.3 |
| `MAX_FILE_SIZE`, `MAX_CLIP_DURATION`, `MAX_LONG_SIDE`, `MAX_INPUT_FPS`, `INPUT_FPS_TOLERANCE` | 16.1 step 1, 6.7 |
| `MIN_WORDS` | 16.4 |
| `OUTPUT_FPS`, `CREATOR_*`, `FREE_*` sizes | 10.4, 11.1, 16.8 |
| 400-line file limit | D-55; exit checklist |
| TS §14.3 one job per worker, cancellation points, transfer rule | 15.6, 16.2-16.8 |
| TS §31 queue sizes (6, 8, 4) and ownership | 16.7, 16.8, 16.9 |
| PS §20.2 budgets | 6.8, 7, 11.8, 16.5, 16.8; measured in 24.1 |

### 3. Decisions that should be copied back into the specs

| Into | Decisions |
|---|---|
| `technicalspec.md` §2, §7 | D-27, D-28, D-58 |
| §5 (file tree) | D-46, D-53, D-55 |
| §10.1 | D-59 |
| §10.8 | D-25 |
| §15.1 | D-34 |
| §16.1, §16.4 | D-37, D-50 |
| §17.1 | D-42, D-47 |
| §19.2, §20.1 | D-29, D-30 |
| §21.1, §21.4 | D-26, D-45 |
| §32 | D-35 |
| §9 C-3, §25.1 P-5 | Section 27 items 12 and 14 |
| `buildplan.md` §4-§8 | The schedule moves D-18, D-20, D-21, D-23, D-24, D-33 |
| `product.md` | None |

## Revision of 2026-10-08: checked against V1 as built

The two parts above were written from the specs and `v1implementation.md`. On 2026-10-08 the document was read against the V1 repository itself (`web/eslint.config.js`, `workers/protocol.ts`, `deny.toml`, the `scripts/`, `ci.yml`) and `docs/v1/v1changelog.md`. What the frozen contracts say held; what V1 did beyond its own plan did not always. The document now has 47 decisions, D-18 to D-64 (D-64 is the reference clip, part 5 below).

### 1. Corrections where the plan did not match V1 as built

| Found | Changed in the document |
|---|---|
| V1's lint matrix lacks three more edges: `models` to `persistence/db`; the types of `workers/protocol.ts` from `state`, `models` and `usecases`; `WorkerCallError` for use-cases | D-27 (now edges a-f), D-32 (`pool.ts` re-exports), D-59 (`state/model-store.ts`), sections 15, 17, 18, 19, 22.3 |
| `offcut-wasm-render` may not depend on `offcut-scene`; the `offcut-text` ban needs `offcut-scene`; third-party parents need wrapper entries | New D-61; D-28; section 22.1 |
| `check-hosts.mjs` reads every byte of `web/dist`, so the render bundle and `ort/` need entries too | D-38; section 22.4; the checks of S6 and S10 |
| `upload-assets.sh` writes one flat level and accepts two folders | New D-62; sections 15.1, 16.5, 17.2, 22.4, 23.3 |
| The asset bucket's CORS policy does not allow the preview port | D-41; sections 3.3, 22.6 |
| No Node types for `tests-e2e`; `bench/` is outside the pnpm workspace | New D-63; sections 3.2, 22.2, 22.3 |
| `CREATOR_VIDEO_BITRATE` already exists in `encoders.ts`; pages take paths as props; the `loadCore()` comment; only `PURE_CRATES` of V1 are checked; CI has three jobs | D-26, D-48, D-36; sections 16.9, 20.1, 22.4, 22.5 |

### 2. Corrections inside the document

| Found | Changed |
|---|---|
| The render bundle and the spike were scheduled before the crates they need | Build order: S9 is now detection and entitlement, S10 the render crates, S11 the spike. TE-3, TE-4 and E-4 refer to S11 |
| The 20,000-sample bound failed at `open`, so a clip that is too long, or one with PCM audio, would have been reported as corrupt | D-60, sections 6.4-6.6 and 6.9: a longer track is left unresolved and `validate_probe` names the reason |
| V2's one visible event depends on the recognizer writing the amount as `$12,000`, with no fallback | D-42, sections 8.4, 8.5, 9.3: a conditional `dollars` form |
| The tree cited section 22.4 for CI; `openDemuxer`'s return type left out the rejection | Section 4; section 13.3 |

### 3. Preconditions

Precondition 10 no longer asks for the `v1` tag or `docs/v1/experiments.md`. The founder decided on 2026-10-08 that V1's code is complete and that its remaining exit boxes (the tag, the experiment records, E-1 outreach, the demo clips) are done later; section 1A lists them with their effect on V2. Precondition 12, the public half of the entitlement key, stays: `config/entitlement-public-key.ts` needs it at S1.

### 4. To copy back into the specs, beyond the table above

| Into | Decisions |
|---|---|
| `technicalspec.md` §2, §7 (web table) | D-27 e and f; D-32's re-export |
| §5 (file tree) | D-63: `web/tests-e2e/tsconfig.json` |
| §15.2, §15.4 | D-60: an unresolved track is not an error |
| §16.1 | D-62: hashed asset names, plain names in OPFS |
| §17.2 | D-42, only if the conditional form is added |
| §7 (crate graph) | Nothing: D-61 keeps the graph as drawn |

### 5. The reference clip (founder's decision, 2026-10-08)

The V2 reference clip is the founder's webcam recording of Script A, not the 60-second 1080x1920 portrait clip of TS §26: `testclips/speech_scriptA_landscape_720p.mp4`, 1280x720, 74.7 s, variable frame rate, 35.2 MB. The reason given: it is what a user really drops in. The clip passes the 11 rules of TS §15.2 unchanged.

| Follows from it | Changed |
|---|---|
| The clip is longer than 60 s | D-64: every PS §20.2 budget and the E-3 and E-4 thresholds are multiplied by 1.245 (transcription 25 s, render and encode 112 s). Readings are recorded as measured and normalised to 60 s. Sections 6.8, 7, 11.8, 16.5, 16.8, 24.1, 24.3 |
| The clip is landscape | The E2E case expects `orientation: "landscape"` and `duration_bucket: "lte90"` (section 23.7). The output is a centre strip 405 pixels wide, enlarged |
| The clip is kept out of git | D-39: it stays in `testclips/`; CI gets it from the asset host, where it is the sample clip (sections 4, 23.4) |
| A portrait clip is still wanted | It is recorded in V3 with the other fixtures, and E-3 and E-4 are read again on it (section 25.2) |

To copy back: TS §26 gains a row for this clip; PS §20.2 is unchanged, since its budgets stay defined for 60 s.