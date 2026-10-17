//! The last stage of the voice chain (TS §18.2): a look-ahead peak limiter at
//! the ceiling, written so that it adds no delay. Each sample gets the gain it
//! needs to stay under the ceiling; the gain is lowered ahead of a peak by
//! reading ahead, never by delaying the signal, so no sample moves in time and
//! the length is unchanged (INV-10).

use std::collections::VecDeque;

use offcut_types::{Dbfs, Hz};

use crate::config::{LIMITER_LOOKAHEAD, LIMITER_RELEASE_MS};

/// Limits `samples` in place to `ceiling` and returns how many samples the
/// final guard clamp had to touch, which is 0 when the gain did its work.
///
/// For each sample the required gain is `r[n] = min(1, c / |x[n]|)`. The attack
/// gain is the moving minimum of `r` over `[n, n + LIMITER_LOOKAHEAD]`, smoothed
/// by a moving average of the same length: it ramps down before a peak and
/// reaches `r` at the peak. The release recovers toward 1 with
/// `exp(-1 / (0.050 x rate))`, never above the attack gain.
pub fn limit(samples: &mut [f32], ceiling: Dbfs, rate: Hz) -> usize {
    let c32 = 10_f32.powf(ceiling.get() / 20.0);
    let c = f64::from(c32);
    let required: Vec<f64> = samples
        .iter()
        .map(|&s| {
            let a = f64::from(s.abs());
            if a > c { c / a } else { 1.0 }
        })
        .collect();
    let attack = attack_gain(&required, LIMITER_LOOKAHEAD);
    let release = (-1.0 / (f64::from(LIMITER_RELEASE_MS) / 1_000.0 * f64::from(rate.get()))).exp();

    let mut gain = 1.0_f64;
    let mut clamped = 0;
    for (s, a) in samples.iter_mut().zip(attack) {
        // `1 - (1 - g) x release` keeps a gain of exactly 1 at exactly 1.
        gain = (1.0 - (1.0 - gain) * release).min(a);
        let y = (f64::from(*s) * gain) as f32;
        *s = if y > c32 {
            clamped += 1;
            c32
        } else if y < -c32 {
            clamped += 1;
            -c32
        } else {
            y
        };
    }
    clamped
}

/// The attack gain of every sample: the mean of `m[k]` over
/// `k in [n - len + 1, n]`, where `m[k]` is the least required gain over
/// `[k, k + len]`. Every such window holds sample `n`, so the result is never
/// above `required[n]`. Before the first sample, `m[k]` is the least required
/// gain over `[0, k + len]`, so a peak near the start is ramped to as well.
fn attack_gain(required: &[f64], len: usize) -> Vec<f64> {
    let n = required.len();
    if n == 0 || len == 0 {
        return required.to_vec();
    }
    let window_min = forward_min(required, len);
    // The values of `m` for k = -(len - 1) .. -1, then for k = 0 .. n - 1.
    let mut prefix_min = Vec::with_capacity(len - 1);
    let mut least = 1.0_f64;
    for r in required.iter().take(len).skip(1) {
        least = least.min(*r);
        prefix_min.push(least);
    }
    // A clip shorter than the window: the missing values are the least of all.
    while prefix_min.len() < len - 1 {
        prefix_min.push(least.min(required.first().copied().unwrap_or(1.0)));
    }
    let first = required.first().copied().unwrap_or(1.0);
    let prefix: Vec<f64> = prefix_min.iter().map(|m| m.min(first)).collect();
    let extended: Vec<f64> = prefix.into_iter().chain(window_min).collect();

    // Moving average of `len` values, as a running sum in a fixed order (P8).
    let mut out = Vec::with_capacity(n);
    let mut sum: f64 = extended.iter().take(len).sum();
    out.push(sum / len as f64);
    for (leaving, entering) in extended.iter().zip(extended.iter().skip(len)) {
        sum += entering - leaving;
        out.push(sum / len as f64);
    }
    out
}

/// `m[k] = min(required[k..=k + len])`, clipped at the end, for every `k`, in
/// one pass from the end with a monotonic queue of indices.
fn forward_min(required: &[f64], len: usize) -> Vec<f64> {
    let mut out = vec![1.0_f64; required.len()];
    let mut queue: VecDeque<(usize, f64)> = VecDeque::new();
    for (k, (&r, slot)) in required.iter().zip(out.iter_mut()).enumerate().rev() {
        while queue.back().is_some_and(|&(_, v)| v >= r) {
            queue.pop_back();
        }
        queue.push_back((k, r));
        while queue.front().is_some_and(|&(i, _)| i > k + len) {
            queue.pop_front();
        }
        *slot = queue.front().map_or(r, |&(_, v)| v);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LIMIT_CEILING_DBFS, SAMPLE_RATE};

    fn ceiling() -> Dbfs {
        Dbfs::new(LIMIT_CEILING_DBFS).unwrap()
    }

    fn peak_dbfs(x: &[f32]) -> f32 {
        20.0 * x.iter().fold(0.0_f32, |m, s| m.max(s.abs())).log10()
    }

    #[test]
    fn under_the_ceiling_is_unchanged_bit_for_bit() {
        let input: Vec<f32> = (0..48_000).map(|i| (i as f32 * 0.01).sin() * 0.8).collect();
        let mut out = input.clone();
        assert_eq!(limit(&mut out, ceiling(), SAMPLE_RATE), 0);
        assert_eq!(out, input);
    }

    #[test]
    fn a_full_scale_square_wave_stays_under_the_ceiling() {
        let mut x: Vec<f32> = (0..48_000)
            .map(|i| if (i / 100) % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        assert_eq!(limit(&mut x, ceiling(), SAMPLE_RATE), 0);
        assert!(peak_dbfs(&x) <= -1.0 + 1e-6, "{}", peak_dbfs(&x));
    }

    #[test]
    fn a_spike_is_ramped_to_from_240_samples_before() {
        let (p, len) = (10_000, LIMITER_LOOKAHEAD);
        let mut x = vec![0.1_f32; 20_000];
        x[p] = 1.0;
        assert_eq!(limit(&mut x, ceiling(), SAMPLE_RATE), 0);
        assert!(peak_dbfs(&x) <= -1.0 + 1e-6);
        // Untouched until the ramp begins; turned down from there to the peak.
        assert_eq!(x[p - len - 1], 0.1);
        assert!(x[p - len / 2] < 0.1);
        // Recovered after it: the release (50 ms, 2,400 samples) leaves under 1% after 10,000 samples.
        assert!((x[19_999] - 0.1).abs() < 1e-3, "{}", x[19_999]);
    }

    #[test]
    fn length_is_unchanged_and_a_spike_at_the_start_is_caught() {
        let mut x = vec![0.0_f32; 1_000];
        x[3] = 1.0;
        assert_eq!(limit(&mut x, ceiling(), SAMPLE_RATE), 0);
        assert_eq!(x.len(), 1_000);
        assert!(peak_dbfs(&x) <= -1.0 + 1e-6);
        let mut short = vec![1.0_f32; 10];
        assert_eq!(limit(&mut short, ceiling(), SAMPLE_RATE), 0);
        assert_eq!(short.len(), 10);
    }
}
