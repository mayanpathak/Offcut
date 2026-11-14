//! The levelled voice (v6.2implementation D-487, TS §47.5; Enhance's voice
//! switched off): the recording brought to the loudness target and held
//! under the ceiling, and nothing else: no high-pass, no denoise, no
//! compressor. It is `run_chain`'s steps 4 to 7 on the input itself, written
//! here so that `run_chain` stays as it is. Same length as the input
//! (INV-10): one gain and the limiter, which adds no delay.

use crate::DspError;
use crate::chain::ChainOutput;
use crate::config::{ChainConfig, LOUDNESS_RETRY_LU, SAMPLE_RATE};
use crate::limiter::limit;
use crate::loudness::{apply_gain, gain_db, integrated, sample_peak};

/// `lvl48` from mono 48 kHz `pcm48`: loudness to `cfg.target` and the limiter
/// at `cfg.ceiling`, with one corrective pass when the first misses by more
/// than `LOUDNESS_RETRY_LU`, as the chain does. `cfg`'s high-pass, denoise and
/// compressor settings are not read. Refuses what `run_chain` refuses.
pub fn level_only(pcm48: &[f32], cfg: &ChainConfig) -> Result<ChainOutput, DspError> {
    if pcm48.is_empty() {
        return Err(DspError::Empty);
    }
    if pcm48.iter().any(|s| !s.is_finite()) {
        return Err(DspError::NonFinite);
    }
    let mut work = pcm48.to_vec();

    // Loudness: one gain to the target, then the limiter.
    let measured = integrated(&work);
    let gain = gain_db(measured, cfg);
    apply_gain(&mut work, gain);
    limit(&mut work, cfg.ceiling, SAMPLE_RATE);

    // One corrective pass at most, only after a real first gain (the
    // programme passed the gate and the gain was not capped).
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

    debug_assert_eq!(work.len(), pcm48.len());
    let loudness = last.unwrap_or(cfg.absolute_silence);
    let peak = sample_peak(&work);
    Ok(ChainOutput {
        samples: work,
        loudness,
        peak,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_refuses_what_the_chain_refuses() {
        let cfg = ChainConfig::default();
        assert_eq!(level_only(&[], &cfg), Err(DspError::Empty));
        assert_eq!(level_only(&[0.1, f32::NAN], &cfg), Err(DspError::NonFinite));
        assert_eq!(
            level_only(&[0.1, f32::INFINITY], &cfg),
            Err(DspError::NonFinite)
        );
    }

    #[test]
    fn silence_stays_silence_at_the_floor() {
        let out = level_only(&[0.0; 48_000], &ChainConfig::default()).unwrap();
        assert!(out.samples.iter().all(|s| *s == 0.0));
        assert_eq!(out.loudness, ChainConfig::default().absolute_silence);
    }
}
