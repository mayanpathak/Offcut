//! The voice chain (TS §18.2, §34.3): high-pass, denoise, compressor in 1 s
//! blocks, then loudness to the target and a limiter at the ceiling, over the
//! whole recording. The output has exactly as many samples as the input, at the
//! same times (INV-10): nothing here detects silence, analyses a gap or a pause,
//! cuts, crossfades, joins segments or stretches time, and no word timing comes
//! in. The only callers are `offcut-wasm-core/src/audio_api.rs` and the tests.

use offcut_types::{Dbfs, Lufs, SampleCount};

use crate::DspError;
use crate::compressor::Compressor;
use crate::config::{BLOCK_SAMPLES, ChainConfig, LOUDNESS_RETRY_LU, SAMPLE_RATE};
use crate::denoise::Denoise;
use crate::highpass::HighPass;
use crate::limiter::limit;
use crate::loudness::{apply_gain, gain_db, integrated, sample_peak};

/// What `on_block` answers after each block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    Continue,
    Cancel,
}

/// The cleaned voice: `samples` is `out48`, as long as the input.
#[derive(Clone, Debug, PartialEq)]
pub struct ChainOutput {
    pub samples: Vec<f32>,
    pub loudness: Lufs,
    pub peak: Dbfs,
}

/// Runs the chain on mono 48 kHz `pcm48`. `on_block` is called once per 1 s
/// block with the samples done so far; `Cancel` returns `Ok(None)` before the
/// next block starts (P7). Postconditions P1-P8 of TS §34.3.
pub fn run_chain(
    pcm48: &[f32],
    cfg: &ChainConfig,
    on_block: &mut dyn FnMut(SampleCount) -> Flow,
) -> Result<Option<ChainOutput>, DspError> {
    // 1. Checks.
    if pcm48.is_empty() {
        return Err(DspError::Empty);
    }
    if pcm48.iter().any(|s| !s.is_finite()) {
        return Err(DspError::NonFinite);
    }

    // 2-3. One working buffer, block by block; each stage keeps its state.
    let mut work = pcm48.to_vec();
    let mut hpf = HighPass::new(cfg.hpf, SAMPLE_RATE);
    let mut comp = Compressor::new(SAMPLE_RATE);
    let mut done: u64 = 0;
    if cfg.denoise {
        let mut den: Vec<f32> = Vec::with_capacity(work.len());
        let mut denoise = Denoise::new(cfg.denoise_mix);
        for block in work.chunks_mut(BLOCK_SAMPLES) {
            hpf.process(block);
            let from = den.len();
            denoise.process(block, &mut den);
            if let Some(new) = den.get_mut(from..) {
                comp.process(new);
            }
            done += block.len() as u64;
            if on_block(SampleCount::new(done)) == Flow::Cancel {
                return Ok(None);
            }
        }
        // 3b. The denoiser's last frame and its delay: every input sample has
        // its output after `finish` (denoise.rs), so the length is the input's.
        let from = den.len();
        denoise.finish(&mut den);
        if let Some(tail) = den.get_mut(from..) {
            comp.process(tail);
        }
        work = den;
    } else {
        for block in work.chunks_mut(BLOCK_SAMPLES) {
            hpf.process(block);
            comp.process(block);
            done += block.len() as u64;
            if on_block(SampleCount::new(done)) == Flow::Cancel {
                return Ok(None);
            }
        }
    }
    debug_assert_eq!(work.len(), pcm48.len());

    // 4. Loudness: one gain to the target.
    let measured = integrated(&work);
    let gain = gain_db(measured, cfg);
    apply_gain(&mut work, gain);
    // 5. Limiter, which adds no delay.
    limit(&mut work, cfg.ceiling, SAMPLE_RATE);

    // 6. Re-measure; one corrective pass at most, only when the first gain was
    // a real one (the programme passed the gate and the gain was not capped).
    let mut last = integrated(&work);
    let gated = measured.is_some_and(|m| m.get() >= cfg.absolute_silence.get());
    let capped = gain >= cfg.max_gain_db;
    if let (true, false, Some(now)) = (gated, capped, last) {
        let off = cfg.target.get() - now.get();
        if off.abs() > LOUDNESS_RETRY_LU {
            apply_gain(&mut work, off);
            limit(&mut work, cfg.ceiling, SAMPLE_RATE);
            last = integrated(&work);
        }
    }

    // 7. Result.
    debug_assert_eq!(work.len(), pcm48.len());
    let loudness = last.unwrap_or(cfg.absolute_silence);
    let peak = sample_peak(&work);
    Ok(Some(ChainOutput {
        samples: work,
        loudness,
        peak,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn go(x: &[f32], cfg: &ChainConfig) -> Result<Option<ChainOutput>, DspError> {
        run_chain(x, cfg, &mut |_| Flow::Continue)
    }

    fn sine(n: usize, peak: f32) -> Vec<f32> {
        (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48_000.0).sin() * peak)
            .collect()
    }

    #[test]
    fn one_sample() {
        let out = go(&[0.25], &ChainConfig::default()).unwrap().unwrap();
        assert_eq!(out.samples.len(), 1);
    }

    #[test]
    fn one_block_and_one_sample() {
        let out = go(&sine(48_001, 0.1), &ChainConfig::default())
            .unwrap()
            .unwrap();
        assert_eq!(out.samples.len(), 48_001);
    }

    #[test]
    fn cancel_at_the_first_call() {
        let mut calls = 0;
        let out = run_chain(&sine(150_000, 0.1), &ChainConfig::default(), &mut |_| {
            calls += 1;
            Flow::Cancel
        });
        assert_eq!(out, Ok(None));
        assert_eq!(calls, 1);
    }

    #[test]
    fn without_denoise_same_length_and_on_target() {
        let cfg = ChainConfig {
            denoise: false,
            ..ChainConfig::default()
        };
        let input = sine(240_000, 10_f32.powf(-30.0 / 20.0));
        let out = go(&input, &cfg).unwrap().unwrap();
        assert_eq!(out.samples.len(), input.len());
        assert!(
            (out.loudness.get() + 14.0).abs() <= 0.5,
            "{}",
            out.loudness.get()
        );
    }
}
