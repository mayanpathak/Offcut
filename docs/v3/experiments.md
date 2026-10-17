# experiments.md - outcome of every experiment V3 runs (v3implementation §20)

Every reading is taken on D1 and recorded under the label `d1` (D-74). One section per row of §20.1, filled when the experiment is run.

## TE-13 native
Question: does the voice chain (`run_chain`, denoise on) run natively in under 2 s per 60 s of audio, the early warning before the browser reading of Prompt 69 (§20.1, G 3.8)?
Method: `chain_timing` of `crates/offcut-dsp/tests/chain_no_clicks.rs`, release build, five runs, on `speech_60s_portrait.mp4.pcm48.f32` (60.0 s, 2,879,992 samples).
Numbers: 0.643, 0.655, 0.707, 0.722, 0.775 s; **median 0.707 s per 60 s** (`d1`). Also read: `measure_prosody` on 60 s with 150 words, 0.148 s in release (budget 2 s, TS §8's 1 s for the reference clip).
Date: 2026-10-17, D1 (`d1`).
Decision: continue. Under 2 s by a factor of 2.8; denoise stays on (`ChainConfig::default()` keeps `denoise: true`). The browser reading (TE-13 browser, at most 4 s per 60 s) is Prompt 69's. A pass on D1 does not show a pass on an R1-class laptop (D-74).

## TE-13 browser
Question: does the voice chain run in the browser, in the audio worker, in at most 4 s per 60 s of audio on D1, with `out48` exactly as long as the input and on target (PS §20.2, §20.2 of the plan, G Phase 4)?
Method: a temporary Playwright check (Chrome stable, D1): the page fetches `speech_60s_portrait.mp4.pcm48.f32` (2,879,992 samples, 60.0 s), calls `pool.audio.measureProsody` once with nothing kept, then three times `pool.audio.runChain({ pcm48 }, { transfer: [pcm48.buffer] })` timed with `performance.now()` from the call to its answer, each followed by `measureProsody({ words: [], sentences: [] })` to take `out48` back. In dev (the Vite dev server, cross-origin isolated by its headers, `/src/workers/pool.ts` imported in the page) and in the production build (`vite preview` with the headers of `vercel.json`, the pool reached through a temporary module imported from `main.tsx`, removed after).
Numbers: dev 866, 705, 733 ms, **median 733 ms per 60 s**; production build 855, 738, 828 ms, median 828 ms (13% over dev; `crossOriginIsolated` true). Every run: `out48.length === pcm48.length` (2,879,992), the input buffer detached by the transfer, loudness -14.30 LUFS. The first `measureProsody` failed with `E_INTERNAL` / `NoAudio`. The worker and its module were started by that first, failing call, so no timed run includes compiling the module; each includes the copy into and out of WASM memory (D-110) and the transfer.
Date: 2026-10-17, D1 (`d1`).
Decision: continue. At most 4,000 ms; the browser is about 1.1 times the native 0.707 s (TE-13 native), not the 1.5-2.5 the plan assumed. Denoise stays on. A pass on D1 does not show a pass on an R1-class laptop (D-74).

## DENOISE_DELAY_SAMPLES
Question: by how many samples does RNNoise (`nnnoiseless` 0.5.2) delay its output, so that `denoise.rs` can align the wet and dry paths and drop exactly that many samples (P2, INV-10)?
Method: `impulses_stay_aligned` of `crates/offcut-dsp/tests/duration_preserved.rs`: 19 impulses of 0.5 on a -40 dBFS noise floor, one every 0.5 s over 10 s; for each, the position of the largest output sample within 2,400 samples of the input impulse, for the denoiser alone at mix 1.0 (all wet, so the dry path cannot hide a misaligned wet one) and for the whole chain. Read with the constant at its planned 480, and again at 0.
Numbers: at 480, all 19 offsets are 0, for the denoiser alone and for the chain; at 0 (the drill), all 19 are +480. The delay is one RNNoise frame, exactly, and the same for every impulse.
Date: 2026-10-17, D1 (native, debug build).
Decision: `DENOISE_DELAY_SAMPLES = 480`, the measured value; denoise stays on (`ChainConfig::default()` keeps `denoise: true`). Native timing on the way: `run_chain` on the 60 s portrait speech PCM took 0.607-0.617 s in a release build (three runs, `chain_timing`); TE-13 is read in Prompt 68.

## Precision gate
Question:
Method:
Numbers:
Date:
Decision:

## TE-12
Question:
Method:
Numbers:
Date:
Decision:

## E-3 re-read
Question:
Method:
Numbers:
Date:
Decision:

## E-4 re-read
Question:
Method:
Numbers:
Date:
Decision:

## E-10 smoke
Question:
Method:
Numbers:
Date:
Decision:

## E-10 (carried to V9, D-88)
Question:
Method:
Numbers:
Date:
Decision:

## rej_h264_444 in Chrome
Question: does Chrome refuse to decode H.264 High 4:4:4 Predictive, so that `rej_h264_444.mp4` tests `REJECT_DECODE_UNSUPPORTED` with a real file (D-78)?
Method: `ffprobe` on the generated file; then `VideoDecoder.isConfigSupported({ codec, codedWidth: 1280, codedHeight: 720 })` in Chrome stable through Playwright on a page of the app's origin, for `avc1.f4001f` (High 4:4:4, level 3.1), D-78's fallback `avc1.6e001f` (High 10) and `avc1.64001f` (High) as a control.
Numbers: the file is `High 4:4:4 Predictive`, level 31. Chrome answers `supported: true` for all three.
Date: 2026-10-17, D1.
Decision: D-78's last branch: both files would decode, so the `REJECT_DECODE_UNSUPPORTED` case of `rejections.spec.ts` is written as `test.fixme` with this reason (Prompt 84), and the code path is covered by the inline unit test of `validate_probe`. The fixture stays, unchanged: `probe_rejections.rs` uses it natively with `decode_supported = false`. No `rej_h264_hi10.mp4` is made (it would decode too). The Windows runner's Chrome is not read here.

## rej_silent
Question: does the recognizer give fewer than `MIN_WORDS` (3) words for 5 s of digital silence, so that `rej_silent.mp4` is refused with `REJECT_NO_SPEECH` (D-111)?
Method: the file dropped in the V2 app (`vite preview` of the current build, model cached in OPFS), through Playwright on D1.
Numbers: the app showed the rejected state (`clip-rejected`).
Date: 2026-10-17, D1.
Decision: the fixture stays at 5 s.

## Derived clips' events
Question: (first reading, Prompt 68) does the measured prosody of "brutal" in the reference speech carry the KeywordPop of Script A, which V3's expected events give as conditional ("if the measured prosody carries it", `fixtures/speech/README.md`, §17.7)?
Method: `the_brutal_reading_of_the_portrait_speech` of `crates/offcut-dsp/tests/prosody.rs`: `measure_prosody` on `speech_60s_portrait.mp4.pcm48.f32` (the uncleaned derived PCM) with sentences 3 and 4 of "The V2 reference clip as recognized", moved 960 ms earlier for the portrait cut. The README gives sentence times only, so the words share their sentence's time by their letters: an approximation, printed and not asserted. The reading with the recognizer's word times on the cleaned voice comes at S8 (Prompt 82).
Numbers: sentence 4 ("This is literally brutal way to make anything."): "literally" energy_z +0.45, pitch_z +1.79; **"brutal" energy_z +0.91, pitch_z +1.26**; the others between -2.12 and +0.91 in energy and -0.95 and +0.50 in pitch. Sentence 3: "3000" energy_z +1.24, pitch_z +0.83.
Date: 2026-10-17, D1.
Decision: none yet. The pitch rise on "literally brutal" is measurable; whether "brutal" passes KeywordPop is read at S8 with real word times and the detector of Prompt 77.

## Second recording
Question:
Method:
Numbers:
Date:
Decision:

## Bundle size
Question: how much does the voice chain (`nnnoiseless`, `ebur128` and `offcut-dsp`'s chain) add to `offcut_core_bg-*.wasm` (G 4.4; growth over 2 MB is the signal to find the crate)?
Method: `pnpm build`, the size of `web/dist/assets/offcut_core_bg-*.wasm`, against Entry 1's baseline.
Numbers: 459,281 bytes at the baseline (V2); 698,122 bytes with `audio_api.rs` exporting `run_chain` and `measure_prosody` (+238,841 bytes, +52%). `offcut_render_bg` unchanged at 6,033,574. The new `audio.worker` chunk: 15,146 bytes. `check-hosts.mjs` finds no new URL literal (the bundle's only host is still the asset host).
Date: 2026-10-17, D1.
Decision: accepted; no crate to look into (well under 2 MB of growth). No entry in `NON_NETWORK_LITERALS`.
