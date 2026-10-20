//! The first stage of the voice chain: a second-order high-pass that removes
//! rumble below the voice (TS §18.2). RBJ cookbook coefficients, Q = `HPF_Q`,
//! Direct Form II transposed with `f64` state carried across blocks, so a block
//! boundary is invisible (P6). It adds no delay that matters to alignment.

use offcut_types::Hz;

use crate::config::HPF_Q;

pub struct HighPass {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    z1: f64,
    z2: f64,
}

impl HighPass {
    /// A cutoff of 0, or one not below `rate / 2`, gives a filter that passes
    /// the signal unchanged.
    pub fn new(cutoff: Hz, rate: Hz) -> HighPass {
        let (f, fs) = (f64::from(cutoff.get()), f64::from(rate.get()));
        if f <= 0.0 || f >= fs / 2.0 {
            return HighPass {
                b0: 1.0,
                b1: 0.0,
                b2: 0.0,
                a1: 0.0,
                a2: 0.0,
                z1: 0.0,
                z2: 0.0,
            };
        }
        let w0 = 2.0 * std::f64::consts::PI * f / fs;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * HPF_Q);
        let a0 = 1.0 + alpha;
        HighPass {
            b0: (1.0 + cos) / 2.0 / a0,
            b1: -(1.0 + cos) / a0,
            b2: (1.0 + cos) / 2.0 / a0,
            a1: -2.0 * cos / a0,
            a2: (1.0 - alpha) / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// Filters `block` in place; the state is kept for the next block.
    pub fn process(&mut self, block: &mut [f32]) {
        for s in block.iter_mut() {
            let x = f64::from(*s);
            let y = self.b0 * x + self.z1;
            self.z1 = self.b1 * x - self.a1 * y + self.z2;
            self.z2 = self.b2 * x - self.a2 * y;
            *s = y as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{HPF_HZ, SAMPLE_RATE};

    fn sine(freq: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / 48_000.0).sin() * 0.5)
            .collect()
    }

    fn rms(x: &[f32]) -> f64 {
        (x.iter().map(|&s| f64::from(s) * f64::from(s)).sum::<f64>() / x.len() as f64).sqrt()
    }

    /// Gain in dB of the filter for a sine, read after the filter has settled.
    fn gain_db(freq: f32) -> f64 {
        let input = sine(freq, 96_000);
        let mut output = input.clone();
        HighPass::new(HPF_HZ, SAMPLE_RATE).process(&mut output);
        20.0 * (rms(&output[48_000..]) / rms(&input[48_000..])).log10()
    }

    #[test]
    fn length_is_unchanged_and_blocks_join_seamlessly() {
        let input = sine(200.0, 10_000);
        let mut whole = input.clone();
        HighPass::new(HPF_HZ, SAMPLE_RATE).process(&mut whole);
        let mut split = input.clone();
        let mut hp = HighPass::new(HPF_HZ, SAMPLE_RATE);
        let (a, b) = split.split_at_mut(3_333);
        hp.process(a);
        hp.process(b);
        assert_eq!(split.len(), input.len());
        assert_eq!(split, whole);
    }

    #[test]
    fn cutoff_zero_or_above_nyquist_passes_unchanged() {
        let input = sine(50.0, 1_000);
        for cutoff in [0, 24_000, 30_000] {
            let mut out = input.clone();
            HighPass::new(Hz::new(cutoff), SAMPLE_RATE).process(&mut out);
            assert_eq!(out, input, "cutoff {cutoff}");
        }
    }

    #[test]
    fn dc_decays_to_zero() {
        let mut dc = vec![0.5_f32; 48_000];
        HighPass::new(HPF_HZ, SAMPLE_RATE).process(&mut dc);
        assert!(dc[47_999].abs() < 1e-6, "{}", dc[47_999]);
    }

    #[test]
    fn one_kilohertz_passes_within_005_db() {
        let g = gain_db(1_000.0);
        assert!(g.abs() < 0.05, "{g} dB");
    }

    #[test]
    fn forty_hertz_is_down_at_least_10_db() {
        let g = gain_db(40.0);
        assert!(g <= -10.0, "{g} dB");
    }

    #[test]
    fn impulse_response_peaks_at_sample_0() {
        let mut x = vec![0.0_f32; 2_000];
        x[0] = 1.0;
        HighPass::new(HPF_HZ, SAMPLE_RATE).process(&mut x);
        let peak = x.iter().enumerate().fold(
            0,
            |best, (i, v)| if v.abs() > x[best].abs() { i } else { best },
        );
        assert_eq!(peak, 0);
    }
}
