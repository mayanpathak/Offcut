# cloudbuildguide.md - Offcut cloud transcription: step-by-step build guide

## What this is

`docs/cloud_transcription/cloudimplementation.md` says what each file contains and why. This guide says what to do next, which command to run, and how to know it worked. The release is built on `cloud-build`, with a draft pull request into `main` so that CI runs. It ends with the pull request merged, the server switch turned on after the checks, the founder's live journey done, every box of the plan's §17 ticked, and the tag `cloud` pushed.

## How to read the references

| Notation | Means |
|---|---|
| `§n` | Section `n` of `cloudimplementation.md` |
| `TS §n`, `PS §n`, `BP §n` | `technicalspec.md`, `product.md`, `buildplan.md` (v1.3) |
| `V6 §n`, `DCL` | `docs/v6/v6implementation.md`; `docs/design/designchangelog.md` |
| `CT CL` | `docs/cloud_transcription/cloudchangelog.md` |
| `D-n` | A decision: D-336 to D-382 are this release's (§2) |
| `F-n` | A founder decision still open (§15) |
| `TE-Cn` | An experiment of §13, recorded in `docs/cloud_transcription/experiments.md` |
| `P<nn>` | A prompt of `coding-promptscloud.md` (161 to 170) |
| Tier 0-4 | The test tiers of Step 0.5 |
| D1 | The founder's Windows machine; readings are labelled `d1` |
| "the founder" | The person running the build. Dashboards (Render, Groq, Vercel), production reads, the merge and the tag are theirs |

## Phase overview

| Phase | Prompt | Covers | Agent time (estimate) | Milestone in one line |
|---|---|---|---|---|
| 0 Setup | Setup | §1A, D-336 to D-341 | 30 min | `cloud-build`, documents committed, `8d94a06` picked, floor read, draft PR |
| 1 Probes | 161 | §13 TE-C1, TE-C2 | 2.5 h + founder (F-1) | Groq's real answers and the browser's Opus encoder read; fixtures recorded; bitrate chosen |
| 2 Server | 162 | §6, §7 (no Groq) | 3 h | The route, its limits, DC-14 and `/healthz` tested with a fake provider |
| 3 Groq | 163 | §7.4, §7.5 | 2.5 h | The adapter from real fixtures; one real call through the local API |
| 4 Encoder | 164 | §8.1, §8.3 | 3 h | A browser-made Ogg Opus file that `ffprobe`, Groq and `ogg.rs` all read |
| 5 Pipeline | 165 | §8.2, §8.4, §8.5 | 3.5 h | Cloud mode and fallback in `run-pipeline`, unit-proven; **push 1** |
| 6 Interface | 166 | §9 | 3 h + founder | The card, the copy, the legal text, the landing wording; the founder approves |
| 7 End to end | 167 | §11.4 | 2.5 h | `cloud-transcribe.spec.ts` and the identity case |
| 8 Privacy clip | 168 | §9.7 | 1 h | The clip re-rendered, uploaded once, wired; **push 2** |
| 9 Experiments | 169 | §13 TE-C3, TE-C4 | 1.5 h + founder | Events and speed, cloud against device; the founder's local walk |
| 10 Close | 170 | §12, §17, §18 | 2.5 h + founder | Copy-back, one full media run, merge, switch on, live checks, tag |

Total about 25 hours of agent time over 3-4 working days, plus the founder's steps. All estimates.

## Read this first: 18 things that will bite you

1. **Never send `testclips/v3testclip.mp4` (it carries a location tag), any `v3testclip*` file, any `WIN_*` recording or any corpus clip to Groq** (D-379, P-13, V3 D-112). Real calls use the reference clip's audio (`fixtures/.gen/speech/speech_60s_portrait.mp4` and friends, or `testclips/speech_scriptA_landscape_720p.mp4`) and synthetic tones only, and only after F-1.
2. **Never print, log, commit or paste the Groq key.** A script that needs it sources `.env.deploy` in a subshell and prints names, status codes and times only. `git grep -nE "gsk_[A-Za-z0-9]{20}"` before every push: no output.
3. **The fake API's `/healthz` must keep answering without `cloud_transcribe` by default.** That absence is what keeps every protected suite on the device (D-366). New specs turn it on through the new option.
4. **The device branch of `run-pipeline.ts` is moved, not edited** (§8.5). Diff it with `git diff -w`: today's step 7-8 lines appear once, inside `hearOnDevice`, unchanged. `run-pipeline.test.ts`'s existing cases must pass without a change.
5. **`media_guard`'s exception is method + exact path + exact media type** (§7.2). `audio/ogg` on any other path is still 415, and so is any other type on `/transcribe`.
6. **Read the limits before the body** (D-352). Take `Request` and call `to_bytes(body, MiB)` yourself after auth, the buckets and the switch; never put a `Bytes` extractor in the handler's arguments, or a stranger's 1 MiB is read before the 401.
7. **Render stops at boot on a missing variable.** The three `TRANSCRIBE_*` values go into Render before the merge (G 10.2), with `TRANSCRIBE_ENABLED=false`.
8. **`asr_cloud` and `asr_backend: "cloud"` are sent only when availability is `"available"`** (D-377). Otherwise an older server rejects the whole analytics batch.
9. **New copy goes in `copy/transcription.ts`.** `messages.ts` is 652 of 700 lines.
10. **The Ogg end must trim exactly.** The last page's granule is pre-skip + `pcm16.length × 3` (48 kHz samples), and `ogg.rs` reads duration from it. A wrong pre-skip shifts every caption (TE-C2 measures the offset).
11. **Media suites run on a keyed build.** `export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=...` (the ci.yml value, read with a non-greedy extract: its line ends in `# gitleaks:allow`) **before** `pnpm build:e2e`, from Git Bash. Without it exports fail with `E_ENTITLEMENT_INVALID`.
12. **Playwright wipes `web/test-results/` at every run.** Copy captures and saved request bodies out (to `~/cloud-review/`) before the next run.
13. **D1's e2e is load-sensitive.** Run with `--workers=4` and rerun a failure alone before believing it.
14. **Never use Vercel's Redeploy button** (it skips the test-key check). A redeploy is a pushed commit.
15. **`target/` grows huge.** Build only the crates a prompt touches (`cargo test -p offcut-api --test transcribe_route`), not the workspace, until P170.
16. **Docs are edited with the Edit tool,** never with Python on Windows (it writes CRLF) and never with a heredoc that holds backticks.
17. **The asset host is append-only.** P168 uploads one file, once. Nothing is deleted or overwritten.
18. **A real Groq call from D1 is the founder's quota too.** Each one is listed in CT CL with its audio's length. The free tier's day is about 28,800 audio seconds; the whole build should use under 1,000.

---

## Phase 0 - Setup

### Step 0.1 - Founder, before the agent starts

- F-1 answered (needed by P161).
- F-2: Groq console → Data Controls → Zero Data Retention on (needed by P166, required before the merge).
- F-3, F-4, F-5 answered (needed by P166 and G 10).

The answers go into CT CL Entry 1, word for word.

### Step 0.2 - The branch

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut
git fetch origin --tags --prune
git rev-parse --short origin/main                 # expect: 68d2ae2
git status --porcelain                            # expect: only ?? docs/cloud_transcription/
git switch -c cloud-build origin/main             # carries the untracked documents
git cherry-pick 8d94a06                           # V6's closing entry (D-340); docs only
git show --stat HEAD | tail -3                    # expect: docs/v6/v6changelog.md only
```

### Step 0.3 - The documents

TS §5 gets five rows for `docs/cloud_transcription/` beside the `docs/design/` rows. `cloudchangelog.md`'s Entry 1 records the branch, the base, the cherry-pick, F-1 to F-5's answers so far, and the floor. Commit `CT setup: the cloud transcription documents`; push to `origin cloud-build`; open the draft pull request:

```bash
gh pr create --draft --base main --head cloud-build --title "Cloud transcription" --body-file <scratchpad file>
```

The body lists the plan's one-sentence goal, DC-14, and the rollout of §12.2, and ends with the attribution line.

### Step 0.4 - The frozen check (every prompt runs it)

```bash
BASE=68d2ae2
git diff --name-only "$BASE" -- fixtures/golden fixtures/labeled fixtures/speech verify web/src/models server/migrations \
  crates/offcut-types crates/offcut-plan crates/offcut-scene crates/offcut-render crates/offcut-text crates/offcut-detect \
  crates/offcut-dsp crates/offcut-mp4 crates/offcut-wasm-core crates/offcut-wasm-render \
  web/src/workers/asr/whisper-runtime.ts web/src/workers/asr/word-timestamps.ts web/src/workers/asr/model-cache-adapter.ts \
  web/src/workers/media web/src/workers/render web/src/workers/media.worker.ts web/src/workers/render.worker.ts \
  web/src/workers/audio.worker.ts web/src/workers/rpc.ts web/src/workers/rpc-worker.ts web/src/workers/pool.ts
# expect: no output, always
git diff --name-only "$BASE" -- '*.snap' '*.png'
# expect: no output, always
```

### Step 0.5 - The tiers

| Tier | Runs | Used by |
|---|---|---|
| 0 | The new or changed case alone | Every prompt, first |
| 1 | `pnpm check`; `cargo test -p` each crate touched (server: the test files touched); Vitest | Every prompt |
| 2 | Tier 1, a keyed `pnpm build:e2e`, and the named cases: `pnpm e2e:media <spec> --grep "<case>" --max-failures=1` (write `[.]`, never a backslash) | 165, 166, 167, 168 |
| 3 | `pnpm check && pnpm fixtures && pnpm test`, `pnpm e2e` | 167 |
| 4 | Tier 3, full `pnpm e2e:media` on a keyed build, once; CI waited for | 170 |

Server tests need Postgres on port 9000, a Windows service on D1, not Docker (V6 D-305).

### Step 0.6 - The floor

```bash
cargo test --workspace 2>&1 | grep -E "^test result" | awk '{p+=$4; f+=$6; i+=$8} END {print p" / "f" / "i}'   # expect 817 / 0 / 4
pnpm --filter web exec vitest run 2>&1 | tail -4                                                              # expect 522
pnpm e2e --workers=4 2>&1 | tail -4                                                                           # expect 57
```

Record the three in CT CL Entry 1. No full media run here (D-339).

**Milestone 0.** `git branch --show-current` prints `cloud-build`; `gh pr view` shows a draft; the counts match D-339 or the entry says why.

---

## Phase 1 - Probes (P161)

Nothing of this phase is product code. Every file is in the scratchpad, every result in `experiments.md` and CT CL.

### Step 1.1 - The audio

```bash
S=<the scratchpad>
REF=fixtures/.gen/speech/speech_60s_portrait.mp4        # pnpm fixtures makes it from the reference clip
ffmpeg -v error -i "$REF" -vn -ac 1 -ar 16000 -c:a flac "$S/ref.flac"
for kbps in 16 24 32; do
  ffmpeg -v error -i "$REF" -vn -ac 1 -ar 16000 -c:a libopus -b:a ${kbps}k "$S/ref${kbps}.ogg"
done
ls -l "$S"/ref*                                          # sizes: 32k about 0.24 MB for 60 s
ffmpeg -v error -f lavfi -i sine=frequency=440:duration=3 -ac 1 -ar 16000 -c:a libopus -b:a 32k "$S/ffmpeg_tone_3s.ogg"
```

### Step 1.2 - Groq, for real (TE-C2; only after F-1)

```bash
groq() {  # $1 = input file, $2 = output json; prints status and time only
  ( set -a; . ./.env.deploy; set +a
    curl -sS https://api.groq.com/openai/v1/audio/transcriptions \
      -H "Authorization: Bearer $TRANSCRIBE_API_KEY" \
      -F "file=@$1" -F model=whisper-large-v3-turbo -F response_format=verbose_json \
      -F "timestamp_granularities[]=word" -F language=en -F temperature=0 \
      -o "$2" -w '%{http_code} %{time_total}s\n' )
}
for f in ref.flac ref16.ogg ref24.ogg ref32.ogg; do groq "$S/$f" "$S/groq-$f.json"; done
```

Then read, and record in `experiments.md`:

1. The top-level fields of a response, and of one entry of `words` (is there `words` at all; `word`, `start`, `end`; anything else).
2. **Whether `words[].word` carries punctuation** and how numbers are written in `words` and in `text` ("3,000" or "3000" or "three thousand"; "$"; "%"). This decides how much of `align.rs` does work (D-360).
3. Word count; sentence count after `.?!`; the 17 sentence starts against "The V2 reference clip as recognized" (`fixtures/speech/README.md`) for the same 60 s window: median absolute difference.
4. Word error rate against Script A's text, for each file; latency; the response's `duration`.
5. Choose the bitrate: the lowest whose WER is within 1 point of FLAC's (expected 32 kbps). Write it into D-370 if it differs, with CT CL saying so.
6. Groq's console: the free tier's limits for this model as the console shows them (requests and audio seconds per hour and per day), and whether Zero Data Retention is on (F-2).

**Stop and ask** (TS §41.4) if: `words` is absent or has no times; the median start difference is over 60 ms; any status is not 200.

### Step 1.3 - The fixtures

Copy the FLAC and 32 kbps responses to `server/tests/fixtures/groq/speech_60s_portrait.flac.json` and `speech_60s_portrait.opus32.json`, with `x_groq.id` (and any other request id) replaced by `"req_fixture"`. Copy `ffmpeg_tone_3s.ogg` to `server/tests/fixtures/ogg/`. Commit them in P162 with their TS §5 rows (fixtures only; no code in P161's commit beyond documents).

### Step 1.4 - The browser's encoder (TE-C1)

A throwaway page (`web/zz-opus-probe.html` and `web/src/zz-opus-probe.ts`, deleted before the commit) built with `pnpm build:e2e` and opened under `pnpm --filter web exec vite preview` in Chrome and in Edge on D1:

1. `AudioEncoder.isConfigSupported` for `{codec: "opus", sampleRate: 16000 | 48000, numberOfChannels: 1, bitrate: 24000 | 32000 | 48000, opus: {frameDuration: 20000}}`: each answer.
2. Encode 60 s of a 440 Hz sine at 16 kHz in 1 s `AudioData`: time taken; chunk count and durations; the first output's `decoderConfig.description` (length; does it start with `OpusHead`; its pre-skip).
3. Record in `experiments.md`. If 16 kHz is unsupported in either browser, D-370's 48 kHz path is built in P164; if Opus is missing: **stop and ask**.

**Milestone 1.** `experiments.md` has TE-C1 and TE-C2 with their numbers; CT CL lists every real Groq call (file, seconds of audio, status); `git status --porcelain` shows no `zz` file. Commit `CT P161: the probes` (documents and the two fixture folders' files).

---

## Phase 2 - The server without Groq (P162)

### Step 2.1 - Types

`offcut-api-types/src/transcribe.rs` (§6.2); `errors.rs` (`TranscribeUnavailable`, 14 codes, the count test); `analytics.rs` (`AsrBackend::Cloud`, `CloudOutcome`, `AnalyticsEvent::AsrCloud { outcome, duration_ms }` with its doc line); `lib.rs`. Then:

```bash
pnpm gen:types && sh scripts/check-gen-clean.sh     # the second fails until web/src/gen is committed with it
```

### Step 2.2 - Config and state

`config.rs`: the three variables (D-356), `TRANSCRIBE_API_BASE_URL` checked as `https://` or `http://localhost:<port>`/`http://127.0.0.1:<port>`, the key redacted in `Debug`; its tests (`complete()` gains the three). `.env` gains the three names with local values (`false`, `unused`, Groq's URL); the agent writes names and these placeholders only. `render.yaml` gains three `sync: false` entries. `state.rs`: `transcriber`, `transcribe_budget`.

### Step 2.3 - The module

`transcribe/{mod,provider,ogg,budget,route}.rs` (§7.1, §7.3, §7.6). `router.rs`: the route with `ip(RouteGroup::TranscribeIp)` and `body(MiB)`; `media_guard`'s exception (§7.2); `/healthz`'s field. `rate_limit.rs`: two groups (D-363). `error.rs`: the new variant. `main.rs`: `UnconfiguredTranscriber` for now. `tests/common/mod.rs`: `FakeTranscriber` (records bytes received; answers a set result), `TRANSCRIBE_ENABLED` per test.

### Step 2.4 - Tests

`transcribe_route.rs` (§11.1); DC-14 in `no_media_routes.rs`; `notify_me.rs`'s one assertion; `log_redaction.rs`'s case; `rate_limit.rs`; `analytics_allowlist.rs`; the unit tests of `ogg.rs`, `budget.rs`, `media_type`. Then:

```bash
cargo test -p offcut-api-types
cargo test -p offcut-api --lib
cargo test -p offcut-api --test transcribe_route --test no_media_routes --test notify_me --test log_redaction --test rate_limit --test analytics_allowlist
```

**Read the DC-14 diff line by line** (`git diff "$BASE" -- server/tests/no_media_routes.rs`) and paste a summary into CT CL: every old assertion still there; the new list; the new route's three cases.

**Milestone 2.** Tier 1; the server answers `/transcribe` with the fake end to end. Commit `CT P162: the transcribe route, its limits and DC-14, without a provider`.

---

## Phase 3 - Groq (P163)

### Step 3.1 - The dependency

`server/Cargo.toml`: `reqwest` features + `multipart`. Then `cargo deny check` and `cargo tree -p offcut-api -e features | grep -c multipart`. If `deny` refuses a crate it brings, remove the feature and write the multipart body by hand (D-358); record which.

### Step 3.2 - The adapter

`transcribe/groq.rs` and `transcribe/align.rs` from P161's fixtures (§7.4, §7.5). `main.rs`: `GroqTranscriber::new(config.clone())?` when enabled. `groq_adapter.rs` (§11.1) against a local mock server bound to `127.0.0.1:0`, with `TRANSCRIBE_API_BASE_URL` pointing at it.

### Step 3.3 - One real call through the real code (F-1)

A temporary test file `server/tests/zz_groq_live.rs`, its one case `#[ignore]`d, builds `GroqTranscriber` from the environment, reads `$S/ref32.ogg`, calls `transcribe()` once, asserts a 200-shaped result (words, duration within 100 ms of 60,000), and writes the `TranscribeResponse` JSON to `web/tests-e2e/fixtures/cloud/speech_60s_portrait.json`:

```bash
( set -a; . ./.env; . ./.env.deploy; TRANSCRIBE_ENABLED=true; S="$S"; set +a
  cargo test -p offcut-api --test zz_groq_live -- --ignored --nocapture 2>&1 | grep -E "^test |test result" )
rm server/tests/zz_groq_live.rs
```

`.env.deploy` holds the key and the real SMTP and billing values but no `DATABASE_URL` or `APP_ORIGIN` (its database is `PROD_DATABASE_URL`, a name nothing reads), so sourcing it after `.env` never points anything at production. The test prints no word of the transcript. CT CL lists the call (60 s of audio). The full route, with a real session, is exercised in P169.

**Milestone 3.** Tier 1; `cargo deny check` clean; one real call recorded; the e2e fixture written by the adapter. Commit `CT P163: the Groq adapter, punctuation, one real call`.

---

## Phase 4 - The encoder (P164)

### Step 4.1 - The flag

`config/features.ts` (§8.1); `vite.config.ts`'s define; `vitest.config.ts` if it needs the define too. TS §5's row for `features.ts` loses "(V6.1)".

### Step 4.2 - The muxer and the encoder

`workers/asr/ogg-opus.ts`, `workers/asr/cloud-audio.ts`, their tests (§11.2); `protocol.ts`'s two members; `asr.worker.ts`'s two handlers; the 48 kHz path only if TE-C1 needs it.

### Step 4.3 - The file is real

A temporary page or Playwright `zz` case in a keyed build: encode the reference clip's `pcm16` (decode `speech_60s_portrait.mp4` with the media worker as the pipeline does, or feed the `.pcm48.f32` file decimated by 3) through `encodeForCloud`, and save the Ogg to the scratchpad. Then:

```bash
ffprobe -v error -show_entries stream=codec_name,channels,sample_rate:format=duration -of default=nw=1 "$S/client_ref.ogg"
# expect codec_name=opus, channels=1, duration within 0.040 s of 60.000
groq "$S/client_ref.ogg" "$S/groq-client_ref.json"     # F-1: one call; expect 200 and a WER like ref32's
```

Encode a 3 s 440 Hz tone the same way and commit it as `server/tests/fixtures/ogg/client_tone_3s.ogg`; `ogg.rs`'s test reads it at 3,000 ms within 20 ms. Delete every `zz` file.

**Milestone 4.** Tier 1; three readers agree on the client's file (`ffprobe`, Groq, `ogg.rs`). Commit `CT P164: features.ts, the Ogg Opus encoder in asr.worker`.

---

## Phase 5 - The pipeline (P165)

### Step 5.1 - The network

`http.ts` (D-373) and its tests; `api-client.ts`: `transcribe(ogg, signal)` through `authed()`, `checkHealth()` (today's `wake()` with the body returned; `wake` stays as a name if other callers use it).

### Step 5.2 - State and choice

`state/cloud-store.ts`, `persistence/prefs-repo.ts`, `persistence/schema.ts`'s `META_KEYS.transcription`, `usecases/set-transcription.ts`; `start-app.ts` step 6 sets availability and loads the choice; `clip-store.ts`'s `asrMode`.

### Step 5.3 - The use-case and the run

`usecases/transcribe-cloud.ts` (§8.4); `run-pipeline.ts` (§8.5); `eslint.config.js`'s edges (`run-pipeline.ts` → `transcribe-cloud.ts`; `start-app.ts` → nothing new if it calls `api-client.ts` directly, which `usecases` may). The toast through `ui-store`'s `pushToast`.

```bash
git diff -w "$BASE" -- web/src/usecases/run-pipeline.ts | less     # the device block appears once, moved, unchanged
pnpm --filter web exec vitest run src/usecases src/net src/state src/persistence
```

### Step 5.4 - The protected suites that touch the pipeline (Tier 2)

```bash
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=...   # item 11
pnpm build:e2e
pnpm e2e:media pipeline-preview.spec.ts --workers=4
pnpm e2e:media model-download.spec.ts cancel.spec.ts --workers=4
```

All pass unchanged; the fake's `/healthz` has no field, so these runs never leave the device path. Push 1, not waited on.

**Milestone 5.** Tier 1 and 2; push 1. Commit `CT P165: cloud mode and fallback in the pipeline`.

---

## Phase 6 - The interface (P166)

### Step 6.1 - The card

`ui/components/TranscriptionChoice.tsx` in both `DropZone` variants (§9.1); styles in `components.module.css` if it stays under 700, otherwise `transcription.module.css` (and its TS §5 row). The design skill, if the agent has one, loads first; V4's tokens and primitives win.

### Step 6.2 - The copy

`copy/transcription.ts` (§9.2); `ProcessingCard`'s line; the two toasts; `messages.settings` rows and `events.asr_cloud` (§9.3); the Privacy Policy (§9.4; its "neither keeps" sentence only after F-2 is confirmed) and Terms (§9.5); `messages.legal.updated`; `copy/landing.ts` (§9.6, F-3's wording).

### Step 6.3 - The landing cases

List every `landing.spec.ts` case that reads one of §9.6's keys or counts "Coming soon" items. Run them; change only a count that §9.6 changes, and record each as a DECLARED CHANGE in CT CL.

### Step 6.4 - Captures

`pnpm screens` with a temporary state that shows the card in all three states (signed in cloud, signed in device, signed out) on the stage and on the landing page, plus the processing card in cloud mode and a fallback toast. Copy to `~/cloud-review/` (item 12).

### Step 6.5 - axe

The card in all three states: 0 violations.

### Step 6.6 - Founder: the approval (F-4)

The founder reads `~/cloud-review/`, `web/src/copy/transcription.ts`, the Privacy Policy and Terms sections, the settings table and the landing diff, and says yes or gives changes. CT CL records the answer word for word.

**Milestone 6.** Tier 2 (the landing cases, `review-edit` not needed: no editor change); the founder's yes. Commit `CT P166: the card, the copy, the legal text, the landing wording`.

---

## Phase 7 - End to end (P167)

`fake-api.ts`'s options (§11.4), defaults unchanged; `cloud-transcribe.spec.ts`'s six cases; `playwright.config.ts`'s `MEDIA_SUITES`. Case 2 saves the request body to the scratchpad and runs `ffprobe` on it. Then:

```bash
pnpm e2e:media cloud-transcribe.spec.ts --workers=1        # the spec alone; about 6-8 min on D1 (assumption)
pnpm check && pnpm fixtures && pnpm test && pnpm e2e --workers=4     # Tier 3
```

**Milestone 7.** Tier 3; the new spec green twice in a row. Commit `CT P167: cloud-transcribe.spec.ts`.

---

## Phase 8 - The privacy clip (P168)

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut-remotion
# edit src/clips/Privacy.tsx (§9.7), then:
npm run render:all -- privacy
ffprobe -v error -show_entries stream=codec_name,pix_fmt,r_frame_rate,width,height:format=duration,size -of default=nw=1 out/offcut-privacy-1280x720.mp4
# expect h264, yuv420p, 30/1, 1280x720, 6 s, under 1.5 MB, and no audio stream
cd ../offcut
( set -a; . ./.env.deploy; set +a; sh scripts/upload-assets.sh media ../offcut-remotion/out/offcut-privacy-1280x720.mp4 )
# prints the new path only
```

`net/asset-fetch.ts`: `LANDING_CLIPS.privacy.path` = the printed path. `ui/assets/landing/privacy.webp` = the new poster (under 80 kB). Read the clip by eye: loops, no cloud, the sentence. Run the landing cases that load clips (L-6). Push 2, not waited on.

**Milestone 8.** The landing page plays the new clip locally; the old file is untouched on the host. Commit `CT P168: the privacy clip without the cloud`.

---

## Phase 9 - Experiments and the founder's walk (P169)

### Step 9.1 - TE-C3 and TE-C4 (F-1)

```bash
# Terminal 1: the local API, with the real Groq key and the real mail relay (never printed)
( set -a; . ./.env; . ./.env.deploy; TRANSCRIBE_ENABLED=true; set +a; cargo run -p offcut-api )
# Terminal 2: the WASM build and the app, proxying /api/v1 to :8080 (vite.config.ts)
pnpm dev
```

The founder signs in once at `http://localhost:5173/account` with their own address: the link arrives by the real relay and points at the local `APP_ORIGIN`. Then, in that browser, for the reference clip and the five derived clips: drop with Cloud, then with On this device; read the transcript and events from IndexedDB through the dev mirror; time drop to `ready`. Record in `experiments.md` (numbers, kinds and figures only; no transcript text beyond what `fixtures/speech/README.md` already publishes). A lost `number_reveal`, `list_reveal` or `from_to` on the reference clip: **stop and ask**. About 6 × 60 s of audio: listed in CT CL.

### Step 9.2 - Founder: the local walk

The founder, on `pnpm dev` with the real key: the three card states; a cloud clip; a device clip; the toggle remembered after a reload; a fallback (the agent stops the local API mid-run); the processing line; the settings table; the Privacy Policy page. Yes or changes, recorded.

**Milestone 9.** TE-C3, TE-C4 recorded; the founder's walk recorded. Commit `CT P169: experiments and the walk`.

---

## Phase 10 - Close (P170)

| Step | Who | What |
|---|---|---|
| 10.1 | Agent | `git grep -nE "TODO|FIXME" -- server/src/transcribe web/src/usecases/transcribe-cloud.ts web/src/workers/asr web/src/ui/components/TranscriptionChoice.tsx`: empty. Copy-back §18 (TS, PS, BP). Tier 4, once. Push 3; `gh pr checks <pr> --watch` in the background |
| 10.2 | **Founder** | Render → `offcut-api` → Environment: `TRANSCRIBE_ENABLED=false`, `TRANSCRIBE_API_KEY` (from `.env.deploy`), `TRANSCRIBE_API_BASE_URL=https://api.groq.com/openai/v1`. Save |
| 10.3 | **Founder** | Groq console: Zero Data Retention on (F-2), confirmed to the agent |
| 10.4 | **Founder** | `gh pr ready`, merge, watch the `main` run (`ci`, `e2e-media`, `deploy-api`, `deploy-web`). A lavapipe SIGSEGV in golden frames or the `openEditor` flake (known issue 3): one re-run of the failed job |
| 10.5 | Agent | `curl -s https://offcut-one.vercel.app/api/v1/healthz` shows the merge's version and `"cloud_transcribe":false`; `E2E_BASE_URL=https://offcut-one.vercel.app pnpm --filter web exec playwright test --project=non-media --grep @smoke`; the live drop zone has no card; headers equal `web/vercel.json` |
| 10.6 | **Founder** | Render: `TRANSCRIBE_ENABLED=true`. Save; wait for "Live" |
| 10.7 | Agent | `/healthz` shows `true`. TE-C5: an unauthenticated 900 kB `audio/ogg` POST to `https://offcut-one.vercel.app/api/v1/transcribe` answers our 401 (`{"code":"unauthorized"...}`), and a 2 MB one our 413, not Vercel's. Record |
| 10.8 | **Founder** | The live journey: sign in; the card shows Cloud; drop the sample clip; captions in seconds; export (Free, 720p, watermark); switch to On this device and drop again (the model downloads if it is not cached); sign out and see the signed-out card. Say what was seen |
| 10.9 | Agent | The closing entry: counts, TE-C5, the founder's journey, what is open. On `cloud-build`; it reaches `main` with the next merge |
| 10.10 | **Founder** (or the agent, if the founder asks) | `git tag -a cloud -m "Cloud transcription" <merge> && git push origin cloud` |

**If anything is wrong after 10.6:** `TRANSCRIBE_ENABLED=false` in Render, then read (§12.3).

**Milestone 10.** §17 ticked or each open box named. V6.1 opens its own branch.

---

## Quick reference

```bash
# Every terminal: Git Bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
export BENCH_DEVICE=d1
BASE=68d2ae2
pnpm check                                             # all static checks, file tree included
cargo test -p offcut-api --test transcribe_route       # one server file (Postgres on 9000)
pnpm gen:types && sh scripts/check-gen-clean.sh        # after any api-types change
pnpm --filter web exec vitest run                      # Vitest
pnpm build:e2e                                         # keyed build for media suites (item 11)
pnpm e2e --workers=4                                   # non-media
pnpm e2e:media <spec> --grep "<case>" --max-failures=1
git grep -nE "gsk_[A-Za-z0-9]{20}"                                     # before every push: no output
```
