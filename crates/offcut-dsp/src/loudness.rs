//! Loudness of the voice chain (TS §18.2): BS.1770 integrated loudness with
//! gating, measured by `ebur128` (v3implementation D-89), and the gain that
//! brings a programme to the target.

use ebur128::{Channel, EbuR128, Mode};
use offcut_types::{Dbfs, Lufs};

use crate::config::{BLOCK_SAMPLES, ChainConfig, NO_PEAK_DBFS, SAMPLE_RATE, dbfs};

/// Integrated loudness of a mono 48 kHz programme, weighted as channel
/// `Center`. `None` when no block passes the gate (silence, or a programme too
/// short or too quiet to measure).
pub fn integrated(samples: &[f32]) -> Option<Lufs> {
    let mut meter = EbuR128::new(1, SAMPLE_RATE.get(), Mode::I).ok()?;
    meter.set_channel(0, Channel::Center).ok()?;
    // In blocks, so that the meter's memory stays bounded.
    for chunk in samples.chunks(BLOCK_SAMPLES) {
        meter.add_frames_f32(chunk).ok()?;
    }
    let lufs = meter.loudness_global().ok()?;
    Lufs::new(lufs as f32)
}

/// `20 log10(max |s|)`; a buffer whose peak is 0 is reported as -200 dBFS.
pub fn sample_peak(samples: &[f32]) -> Dbfs {
    let peak = samples.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
    if peak > 0.0 {
        dbfs(20.0 * peak.log10())
    } else {
        dbfs(NO_PEAK_DBFS)
    }
}

/// The gain that brings `measured` to `cfg.target`: 0 dB for a programme that
/// could not be measured or is below `cfg.absolute_silence`; otherwise
/// `target - measured`, at most `+cfg.max_gain_db`, with no limit on turning down.
pub fn gain_db(measured: Option<Lufs>, cfg: &ChainConfig) -> f32 {
    match measured {
        Some(m) if m.get() >= cfg.absolute_silence.get() => {
            (cfg.target.get() - m.get()).min(cfg.max_gain_db)
        }
        Some(_) | None => 0.0,
    }
}

/// Multiplies every sample by `db` decibels.
pub fn apply_gain(samples: &mut [f32], db: f32) {
    let factor = 10_f32.powf(db / 20.0);
    for s in samples.iter_mut() {
        *s *= factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ChainConfig {
        ChainConfig::default()
    }

    #[test]
    fn a_1khz_sine_of_peak_minus_20_reads_minus_23() {
        let peak = 10_f32.powf(-20.0 / 20.0);
        let x: Vec<f32> = (0..480_000)
            .map(|i| (2.0 * std::f32::consts::PI * 1_000.0 * i as f32 / 48_000.0).sin() * peak)
            .collect();
        let l = integrated(&x).map(|l| l.get());
        assert!(l.is_some_and(|l| (l + 23.0).abs() <= 0.1), "{l:?}");
    }

    #[test]
    fn digital_silence_is_none() {
        assert_eq!(integrated(&vec![0.0; 480_000]), None);
        assert_eq!(sample_peak(&[0.0; 10]).get(), -200.0);
    }

    #[test]
    fn gain_is_capped_upward() {
        assert_eq!(gain_db(Lufs::new(-50.0), &cfg()), 24.0);
    }

    #[test]
    fn below_absolute_silence_gets_no_gain() {
        assert_eq!(gain_db(Lufs::new(-80.0), &cfg()), 0.0);
        assert_eq!(gain_db(None, &cfg()), 0.0);
    }

    #[test]
    fn a_loud_programme_is_turned_down() {
        assert_eq!(gain_db(Lufs::new(-10.0), &cfg()), -4.0);
    }
}
