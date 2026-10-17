//! How each word was said (TS §18.2; v3implementation §7.9, D-96, D-108): its
//! loudness and its pitch, as z-scores against the other words of its
//! sentence. A measurement the detector reads (KeywordPop, the NumberReveal
//! energy bonus) and nothing more: it returns no audio, changes no sample,
//! reads only the samples inside each word, and decides nothing.

use offcut_types::{Prosody, Sentence, Word, WordProsody};

use crate::config::{
    ENERGY_FLOOR_DB, FIRST_PEAK_SHARE, PITCH_FRAME, PITCH_HOP, PITCH_MAX_LAG, PITCH_MIN_LAG,
    PROSODY_DECIMATE, PROSODY_MIN_WORDS, SAMPLE_RATE, VOICED_FRAME_MIN_DB, VOICING_MIN,
};

/// Samples per millisecond of `out48`.
fn per_ms() -> usize {
    SAMPLE_RATE.get() as usize / 1_000
}

/// The rate of the decimated copy the pitch is measured on: 8 kHz.
fn pitch_rate() -> f64 {
    f64::from(SAMPLE_RATE.get()) / PROSODY_DECIMATE as f64
}

/// Per-word energy and pitch z-scores; `per_word.len() == words.len()`.
pub fn measure_prosody(out48: &[f32], words: &[Word], sentences: &[Sentence]) -> Prosody {
    let measured: Vec<(f64, Option<f64>)> = words.iter().map(|w| measure_word(out48, w)).collect();
    let mut per_word: Vec<WordProsody> = words
        .iter()
        .map(|_| WordProsody {
            energy_z: 0.0,
            pitch_z: 0.0,
        })
        .collect();
    for s in sentences {
        let from = (s.words.start.get() as usize).min(words.len());
        let to = (s.words.end.get() as usize).min(words.len()).max(from);
        let (Some(values), Some(out)) = (measured.get(from..to), per_word.get_mut(from..to)) else {
            continue;
        };
        let energy: Vec<Option<f64>> = values.iter().map(|&(e, _)| Some(e)).collect();
        let pitch: Vec<Option<f64>> = values.iter().map(|&(_, f0)| f0.map(f64::log2)).collect();
        for ((wp, ez), pz) in out.iter_mut().zip(z_scores(&energy)).zip(z_scores(&pitch)) {
            wp.energy_z = ez as f32;
            wp.pitch_z = pz as f32;
        }
    }
    Prosody { per_word }
}

/// A word's energy in dB, and its pitch in Hz when it has voiced frames.
fn measure_word(out48: &[f32], w: &Word) -> (f64, Option<f64>) {
    let start = (w.start_ms.get() as usize)
        .saturating_mul(per_ms())
        .min(out48.len());
    let end = (w.end_ms.get() as usize)
        .saturating_mul(per_ms())
        .min(out48.len());
    let samples = out48.get(start..end).unwrap_or(&[]);
    if samples.is_empty() {
        return (f64::from(ENERGY_FLOOR_DB), None);
    }
    let mean_square = samples
        .iter()
        .map(|&s| f64::from(s) * f64::from(s))
        .sum::<f64>()
        / samples.len() as f64;
    (db(mean_square), pitch(samples))
}

/// `10 log10` of a mean square, floored at `ENERGY_FLOOR_DB`.
fn db(mean_square: f64) -> f64 {
    if mean_square > 0.0 {
        (10.0 * mean_square.log10()).max(f64::from(ENERGY_FLOOR_DB))
    } else {
        f64::from(ENERGY_FLOOR_DB)
    }
}

/// The median F0 of the word's voiced frames (the lower middle for an even
/// count), measured on a copy decimated to 8 kHz (D-108).
fn pitch(samples: &[f32]) -> Option<f64> {
    let (groups, _) = samples.as_chunks::<PROSODY_DECIMATE>();
    let low: Vec<f64> = groups
        .iter()
        .map(|g| g.iter().map(|&s| f64::from(s)).sum::<f64>() / PROSODY_DECIMATE as f64)
        .collect();
    let mut frame = vec![0.0_f64; PITCH_FRAME];
    let mut r = vec![0.0_f64; PITCH_MAX_LAG - PITCH_MIN_LAG + 1];
    let mut f0s: Vec<f64> = Vec::new();
    let mut at = 0;
    while let Some(window) = low.get(at..at + PITCH_FRAME) {
        at += PITCH_HOP;
        let mean = window.iter().sum::<f64>() / PITCH_FRAME as f64;
        for (f, &x) in frame.iter_mut().zip(window) {
            *f = x - mean;
        }
        let power = frame.iter().map(|x| x * x).sum::<f64>() / PITCH_FRAME as f64;
        if db(power) < f64::from(VOICED_FRAME_MIN_DB) {
            continue;
        }
        if let Some(f0) = frame_f0(&frame, &mut r) {
            f0s.push(f0);
        }
    }
    f0s.sort_by(f64::total_cmp);
    f0s.get(f0s.len().saturating_sub(1) / 2).copied()
}

/// The F0 of one mean-removed frame, or `None` when it is not voiced.
fn frame_f0(frame: &[f64], r: &mut [f64]) -> Option<f64> {
    for (lag, slot) in (PITCH_MIN_LAG..=PITCH_MAX_LAG).zip(r.iter_mut()) {
        let (mut xy, mut xx, mut yy) = (0.0_f64, 0.0_f64, 0.0_f64);
        for (x, y) in frame.iter().zip(frame.iter().skip(lag)) {
            xy += x * y;
            xx += x * x;
            yy += y * y;
        }
        *slot = if xx > 0.0 && yy > 0.0 {
            xy / (xx * yy).sqrt()
        } else {
            0.0
        };
    }
    let best = r.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if best < f64::from(VOICING_MIN) {
        return None;
    }
    // The first local peak that comes within FIRST_PEAK_SHARE of the best.
    let bar = best * FIRST_PEAK_SHARE;
    let peak = (0..r.len()).find(|&i| {
        let here = r.get(i).copied().unwrap_or(0.0);
        let before = i
            .checked_sub(1)
            .and_then(|j| r.get(j))
            .copied()
            .unwrap_or(f64::NEG_INFINITY);
        let after = r.get(i + 1).copied().unwrap_or(f64::NEG_INFINITY);
        here >= bar && here >= f64::from(VOICING_MIN) && here >= before && here >= after
    })?;
    // Parabolic interpolation over the peak and its two neighbours.
    let at = |i: Option<usize>| i.and_then(|i| r.get(i)).copied();
    let refined = match (at(peak.checked_sub(1)), at(Some(peak)), at(Some(peak + 1))) {
        (Some(a), Some(b), Some(c)) if (a - 2.0 * b + c).abs() > 1e-12 => {
            let shift = 0.5 * (a - c) / (a - 2.0 * b + c);
            peak as f64 + shift.clamp(-0.5, 0.5)
        }
        _ => peak as f64,
    };
    Some(pitch_rate() / (PITCH_MIN_LAG as f64 + refined))
}

/// Population z-scores of the values present; 0.0 for an absent value, and
/// for every value when fewer than `PROSODY_MIN_WORDS` are present or their
/// standard deviation is under 1e-6.
fn z_scores(values: &[Option<f64>]) -> Vec<f64> {
    let present: Vec<f64> = values.iter().flatten().copied().collect();
    if present.len() < PROSODY_MIN_WORDS {
        return vec![0.0; values.len()];
    }
    let n = present.len() as f64;
    let mean = present.iter().sum::<f64>() / n;
    let sd = (present.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / n).sqrt();
    if sd < 1e-6 {
        return vec![0.0; values.len()];
    }
    values
        .iter()
        .map(|v| v.map_or(0.0, |v| (v - mean) / sd))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use offcut_types::{Confidence, TimeMs, WordIdx, WordRange};

    /// Words of `ms` milliseconds with 100 ms of silence between them, one sentence.
    fn layout(n: usize, ms: u32) -> (Vec<Word>, Vec<Sentence>) {
        let words: Vec<Word> = (0..n as u32)
            .map(|i| Word {
                text: format!("w{i}"),
                start_ms: TimeMs::new(100 + i * (ms + 100)),
                end_ms: TimeMs::new(100 + i * (ms + 100) + ms),
                confidence: Confidence::new(1.0).unwrap(),
            })
            .collect();
        let end = words.last().map_or(0, |w| w.end_ms.get());
        let sentence = Sentence {
            words: WordRange {
                start: WordIdx::new(0),
                end: WordIdx::new(n as u32),
            },
            start_ms: TimeMs::new(0),
            end_ms: TimeMs::new(end),
        };
        (words, vec![sentence])
    }

    /// A sawtooth of `f0` at `peak` under each word.
    fn render(words: &[Word], tones: &[(f32, f32)]) -> Vec<f32> {
        let end = words
            .last()
            .map_or(0, |w| w.end_ms.get() as usize * 48 + 4_800);
        let mut x = vec![0.0_f32; end];
        for (w, &(f0, peak)) in words.iter().zip(tones) {
            let (a, b) = (w.start_ms.get() as usize * 48, w.end_ms.get() as usize * 48);
            for (i, s) in x[a..b].iter_mut().enumerate() {
                let phase = (i as f32 * f0 / 48_000.0).fract();
                *s = (2.0 * phase - 1.0) * peak;
            }
        }
        x
    }

    #[test]
    fn zero_words() {
        assert!(measure_prosody(&[0.0; 100], &[], &[]).per_word.is_empty());
    }

    #[test]
    fn a_louder_word_has_the_largest_energy_z() {
        let (words, sentences) = layout(4, 300);
        let x = render(
            &words,
            &[(120.0, 0.1), (120.0, 0.1), (120.0, 0.4), (120.0, 0.1)],
        );
        let p = measure_prosody(&x, &words, &sentences).per_word;
        let e: Vec<f32> = p.iter().map(|w| w.energy_z).collect();
        assert!(e[2] > 1.0 && e.iter().all(|&v| v <= e[2]), "{e:?}");
    }

    #[test]
    fn a_higher_word_has_the_largest_pitch_z() {
        let (words, sentences) = layout(4, 300);
        let x = render(
            &words,
            &[(120.0, 0.1), (120.0, 0.1), (200.0, 0.1), (120.0, 0.1)],
        );
        let p = measure_prosody(&x, &words, &sentences).per_word;
        let z: Vec<f32> = p.iter().map(|w| w.pitch_z).collect();
        assert!(z[2] > 1.0 && z.iter().all(|&v| v <= z[2]), "{z:?}");
    }

    #[test]
    fn a_sentence_of_two_words_is_all_zero() {
        let (words, sentences) = layout(2, 300);
        let x = render(&words, &[(120.0, 0.1), (200.0, 0.5)]);
        for w in measure_prosody(&x, &words, &sentences).per_word {
            assert_eq!((w.energy_z, w.pitch_z), (0.0, 0.0));
        }
    }

    #[test]
    fn the_same_inputs_give_the_same_prosody() {
        let (words, sentences) = layout(5, 250);
        let x = render(
            &words,
            &[
                (100.0, 0.1),
                (130.0, 0.2),
                (160.0, 0.1),
                (190.0, 0.3),
                (220.0, 0.1),
            ],
        );
        assert_eq!(
            measure_prosody(&x, &words, &sentences),
            measure_prosody(&x, &words, &sentences)
        );
    }
}
