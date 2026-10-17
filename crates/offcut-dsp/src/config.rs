//! Every constant of the voice chain and of the prosody measure, and the one
//! place they are gathered for a run: [`ChainConfig`] (TS §18.3, §34.3). The
//! values are (assumption, M1.2) unless marked derived; E-2 reviews them by ear.
//!
//! Nothing here is a silence threshold for cutting, a minimum pause, a kept
//! pause length or a crossfade: the chain never changes the length of the
//! recording (TS §18.3, INV-5, INV-10). `ABSOLUTE_SILENCE_LUFS` only keeps the
//! loudness step from raising a programme it cannot measure.

use offcut_types::{Dbfs, Hz, Lufs};

pub const SAMPLE_RATE: Hz = Hz::new(48_000); // TS §18 (derived)
pub const BLOCK_SAMPLES: usize = 48_000; // 1 s blocks (TS §18.2)
pub const TARGET_LOUDNESS: f32 = -14.0; // LUFS (TS §18.3, PS §20.1)
pub const LIMIT_CEILING_DBFS: f32 = -1.0;
pub const MAX_GAIN_DB: f32 = 24.0;
pub const ABSOLUTE_SILENCE_LUFS: f32 = -70.0;
pub const LOUDNESS_RETRY_LU: f32 = 0.5; // TS §18.2 step 6
pub const HPF_HZ: Hz = Hz::new(80);
pub const HPF_Q: f64 = std::f64::consts::FRAC_1_SQRT_2; // Butterworth (derived)
pub const DENOISE_MIX: f32 = 0.85;
pub const DENOISE_FRAME: usize = 480; // RNNoise frame at 48 kHz (derived)
pub const DENOISE_SCALE: f32 = 32_768.0; // RNNoise works on 16-bit-scaled floats (derived)
pub const DENOISE_DELAY_SAMPLES: usize = 480; // (assumption, measured at S4 by impulses_stay_aligned)
pub const COMP_THRESHOLD_DB: f32 = -18.0;
pub const COMP_RATIO: f32 = 3.0;
pub const COMP_ATTACK_MS: f32 = 5.0;
pub const COMP_RELEASE_MS: f32 = 80.0;
pub const COMP_KNEE_DB: f32 = 6.0;
pub const LIMITER_LOOKAHEAD: usize = 240; // 5 ms at 48 kHz
pub const LIMITER_RELEASE_MS: f32 = 50.0;
pub const ALIGN_TOLERANCE: usize = 48; // 1 ms (TS §18.3)
pub const CLICK_FACTOR: f32 = 2.0; // P6: a step at a block boundary <= 2 x local RMS
pub const CLICK_RMS_WINDOW: usize = 480; // local RMS over 10 ms either side of the boundary
// Prosody (v3implementation D-96, D-108)
pub const PROSODY_DECIMATE: usize = 6; // 48 kHz -> 8 kHz
pub const PITCH_FRAME: usize = 320; // 40 ms at 8 kHz
pub const PITCH_HOP: usize = 80; // 10 ms at 8 kHz
pub const PITCH_MIN_LAG: usize = 20; // 400 Hz at 8 kHz
pub const PITCH_MAX_LAG: usize = 134; // 59.7 Hz at 8 kHz
pub const VOICING_MIN: f32 = 0.45;
pub const VOICED_FRAME_MIN_DB: f32 = -60.0;
pub const ENERGY_FLOOR_DB: f32 = -100.0;
pub const PROSODY_MIN_WORDS: usize = 3; // TS §18.2

/// The peak reported for a buffer whose peak is 0, which has no finite dB value
/// (v3implementation §7.6). Not one of the 32 tuning constants: a reporting value.
pub(crate) const NO_PEAK_DBFS: f32 = -200.0;

/// The settings of one run of the chain (TS §34.3). The binding takes none:
/// it runs `ChainConfig::default()`, and this is the one place where TE-13's
/// fallback, `denoise: false`, would be written (v3implementation D-110).
#[derive(Clone, Debug, PartialEq)]
pub struct ChainConfig {
    pub denoise: bool,
    pub hpf: Hz,
    pub denoise_mix: f32,
    pub target: Lufs,
    pub ceiling: Dbfs,
    pub max_gain_db: f32,
    pub absolute_silence: Lufs,
}

impl Default for ChainConfig {
    fn default() -> Self {
        ChainConfig {
            denoise: true,
            hpf: HPF_HZ,
            denoise_mix: DENOISE_MIX,
            target: lufs(TARGET_LOUDNESS),
            ceiling: dbfs(LIMIT_CEILING_DBFS),
            max_gain_db: MAX_GAIN_DB,
            absolute_silence: lufs(ABSOLUTE_SILENCE_LUFS),
        }
    }
}

/// A `Lufs` of a finite constant. `Lufs::new` refuses only a value that is not
/// finite, which no constant above is; the fallback 0.0 is finite, so this
/// always ends, and no `unwrap` is needed.
pub(crate) fn lufs(v: f32) -> Lufs {
    Lufs::new(v).unwrap_or_else(|| lufs(0.0))
}

/// As [`lufs`], for a `Dbfs`.
pub(crate) fn dbfs(v: f32) -> Dbfs {
    Dbfs::new(v).unwrap_or_else(|| dbfs(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_the_spec() {
        let c = ChainConfig::default();
        assert!(c.denoise);
        assert_eq!(c.hpf.get(), 80);
        assert_eq!(c.denoise_mix, 0.85);
        assert_eq!(c.target.get(), -14.0);
        assert_eq!(c.ceiling.get(), -1.0);
        assert_eq!(c.max_gain_db, 24.0);
        assert_eq!(c.absolute_silence.get(), -70.0);
    }
}
