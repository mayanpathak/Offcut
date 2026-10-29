//! How loud the cleaned voice is, every 10 ms (TS §40.6): the input of the
//! time map V6.2 makes. It measures; it decides nothing. Nothing calls it in
//! V4 (v4implementation D-163).

use offcut_types::SilenceProfile;

/// 10 ms at 48 kHz.
pub const WINDOW_SAMPLES: usize = 480;
/// What a window of zeros, or one quieter than this, reads.
pub const FLOOR_DB: f32 = -100.0;

/// The RMS level of each 10 ms window of `out48`, in dB of full scale, at
/// least `FLOOR_DB`. A last window shorter than 10 ms is measured over the
/// samples it has.
pub fn silence_profile(out48: &[f32]) -> SilenceProfile {
    let rms_db_10ms = out48
        .chunks(WINDOW_SAMPLES)
        .map(|window| {
            let energy: f64 = window.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
            let mean = energy / window.len() as f64;
            let db = if mean > 0.0 {
                (10.0 * mean.log10()) as f32
            } else {
                FLOOR_DB
            };
            db.max(FLOOR_DB)
        })
        .collect();
    SilenceProfile { rms_db_10ms }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_is_10_ms_and_silence_reads_the_floor() {
        assert!(silence_profile(&[]).rms_db_10ms.is_empty());
        // 25 ms: two whole windows and a short one.
        let profile = silence_profile(&[0.0; 1_200]);
        assert_eq!(profile.rms_db_10ms, [FLOOR_DB; 3]);
        // A constant 0.5 is -6.02 dB in every window, the short one too.
        let half = silence_profile(&[0.5; 1_000]);
        assert_eq!(half.rms_db_10ms.len(), 3);
        for db in half.rms_db_10ms {
            assert!((db - -6.0206).abs() < 1e-3, "{db}");
        }
        // Quieter than the floor reads the floor.
        assert_eq!(silence_profile(&[1e-7; 480]).rms_db_10ms, [FLOOR_DB]);
    }
}
