//! The compressor of the voice chain (TS §18.2): it evens out the voice's
//! level. A soft knee in the log domain on a peak follower in dB, with no
//! look-ahead, no delay and no make-up gain: the gain it applies is never
//! above 1, and raising the level is the loudness stage's job.

use offcut_types::Hz;

use crate::config::{COMP_ATTACK_MS, COMP_KNEE_DB, COMP_RATIO, COMP_RELEASE_MS, COMP_THRESHOLD_DB};

pub struct Compressor {
    env_db: f32,
    attack: f32,
    release: f32,
    threshold: f32,
    ratio: f32,
    knee: f32,
}

impl Compressor {
    pub fn new(rate: Hz) -> Compressor {
        let rate = rate.get() as f32;
        Compressor {
            env_db: level_db(0.0),
            attack: (-1.0 / (COMP_ATTACK_MS / 1_000.0 * rate)).exp(),
            release: (-1.0 / (COMP_RELEASE_MS / 1_000.0 * rate)).exp(),
            threshold: COMP_THRESHOLD_DB,
            ratio: COMP_RATIO,
            knee: COMP_KNEE_DB,
        }
    }

    /// Compresses `block` in place; the detector's state is kept for the next block.
    pub fn process(&mut self, block: &mut [f32]) {
        for s in block.iter_mut() {
            let level = level_db(*s);
            let coeff = if level > self.env_db {
                self.attack
            } else {
                self.release
            };
            self.env_db = coeff * self.env_db + (1.0 - coeff) * level;
            let gain_db = self.curve(self.env_db) - self.env_db;
            let gain = 10_f32.powf(gain_db / 20.0).min(1.0);
            *s *= gain;
        }
    }

    /// The output level for an input level `x`, in dB.
    fn curve(&self, x: f32) -> f32 {
        let (t, w, r) = (self.threshold, self.knee, self.ratio);
        if x <= t - w / 2.0 {
            x
        } else if x >= t + w / 2.0 {
            t + (x - t) / r
        } else {
            let d = x - t + w / 2.0;
            x + (1.0 / r - 1.0) * d * d / (2.0 * w)
        }
    }
}

/// A sample's level in dB, `20 log10(max(|s|, 1e-9))` (v3implementation §7.5).
/// The detector starts from the level of silence.
fn level_db(s: f32) -> f32 {
    20.0 * s.abs().max(1e-9).log10()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SAMPLE_RATE;

    fn sine(peak: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48_000.0).sin() * peak)
            .collect()
    }

    #[test]
    fn under_the_knee_is_unchanged() {
        // -30 dBFS peak: the follower never reaches T - W/2 = -21 dB.
        let input = sine(10_f32.powf(-30.0 / 20.0), 48_000);
        let mut out = input.clone();
        Compressor::new(SAMPLE_RATE).process(&mut out);
        assert_eq!(out, input);
    }

    #[test]
    fn gain_is_never_above_one() {
        let input = sine(0.9, 48_000);
        let mut out = input.clone();
        Compressor::new(SAMPLE_RATE).process(&mut out);
        for (y, x) in out.iter().zip(&input) {
            assert!(y.abs() <= x.abs(), "{y} > {x}");
        }
        // A loud sine is turned down once the follower has settled.
        let late_peak = out[24_000..].iter().fold(0.0_f32, |m, v| m.max(v.abs()));
        assert!(late_peak < 0.9 * 0.7, "{late_peak}");
    }

    #[test]
    fn length_is_unchanged() {
        let mut out = sine(0.5, 12_345);
        Compressor::new(SAMPLE_RATE).process(&mut out);
        assert_eq!(out.len(), 12_345);
    }

    #[test]
    fn the_knee_is_continuous() {
        let c = Compressor::new(SAMPLE_RATE);
        let (lo, hi) = (
            COMP_THRESHOLD_DB - COMP_KNEE_DB / 2.0,
            COMP_THRESHOLD_DB + COMP_KNEE_DB / 2.0,
        );
        assert!((c.curve(lo) - lo).abs() < 1e-4);
        assert!(
            (c.curve(hi) - (COMP_THRESHOLD_DB + (hi - COMP_THRESHOLD_DB) / COMP_RATIO)).abs()
                < 1e-4
        );
    }
}
