# experiments.md - outcome of every experiment V3 runs (v3implementation §20)

Every reading is taken on D1 and recorded under the label `d1` (D-74). One section per row of §20.1, filled when the experiment is run.

## TE-13 native
Question:
Method:
Numbers:
Date:
Decision:

## TE-13 browser
Question:
Method:
Numbers:
Date:
Decision:

## DENOISE_DELAY_SAMPLES
Question:
Method:
Numbers:
Date:
Decision:

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
Question:
Method:
Numbers:
Date:
Decision:

## Second recording
Question:
Method:
Numbers:
Date:
Decision:

## Bundle size
Question:
Method:
Numbers:
Date:
Decision:
