//! The denoiser of the voice chain (TS §18.2): RNNoise in pure Rust
//! (`nnnoiseless`, v3implementation D-89), mixed with the dry signal.
//!
//! RNNoise works on 480-sample frames and its output lags its input. Here the
//! dry signal is delayed by the same `DENOISE_DELAY_SAMPLES` before the two are
//! mixed, and the first `DENOISE_DELAY_SAMPLES` mixed samples are dropped, so
//! output sample `k` is input sample `k`. Every input sample gets exactly one
//! output sample: the length never changes (INV-10). The voice-activity value
//! RNNoise returns is discarded: it decides nothing (TS §36).

use std::collections::VecDeque;

use nnnoiseless::DenoiseState;

use crate::config::{DENOISE_DELAY_SAMPLES, DENOISE_FRAME, DENOISE_SCALE};

pub struct Denoise {
    state: Box<DenoiseState<'static>>,
    /// Input not yet a whole frame (fewer than `DENOISE_FRAME` samples).
    pending: Vec<f32>,
    /// The dry path, delayed by `DENOISE_DELAY_SAMPLES` to meet the wet one.
    dry: VecDeque<f32>,
    mix: f32,
    /// Mixed samples still to drop: the delay, once, at the start.
    skip: usize,
    /// Input samples given to `process`, and output samples appended.
    consumed: usize,
    produced: usize,
    /// While `finish` runs: how many output samples are owed in all.
    limit: Option<usize>,
    wet_in: Vec<f32>,
    wet_out: Vec<f32>,
}

impl Denoise {
    pub fn new(mix: f32) -> Denoise {
        Denoise {
            state: DenoiseState::new(),
            pending: Vec::with_capacity(DENOISE_FRAME),
            dry: std::iter::repeat_n(0.0, DENOISE_DELAY_SAMPLES).collect(),
            mix,
            skip: DENOISE_DELAY_SAMPLES,
            consumed: 0,
            produced: 0,
            limit: None,
            wet_in: vec![0.0; DENOISE_FRAME],
            wet_out: vec![0.0; DENOISE_FRAME],
        }
    }

    /// Consumes `input` and appends the finished output to `out`. The output
    /// lags the input by the samples still pending; `finish` delivers them.
    pub fn process(&mut self, input: &[f32], out: &mut Vec<f32>) {
        for &s in input {
            self.consumed += 1;
            self.push(s, out);
        }
    }

    /// Pushes zeros through until every input sample has its output: after
    /// `finish`, the total appended to `out` equals the total consumed. Called once.
    pub fn finish(&mut self, out: &mut Vec<f32>) {
        let owed = self.consumed;
        self.limit = Some(owed);
        while self.produced < owed {
            self.push(0.0, out);
        }
    }

    fn push(&mut self, s: f32, out: &mut Vec<f32>) {
        self.pending.push(s);
        if self.pending.len() == DENOISE_FRAME {
            self.run_frame(out);
        }
    }

    /// One whole frame through RNNoise, mixed with the delayed dry samples.
    fn run_frame(&mut self, out: &mut Vec<f32>) {
        for (w, &s) in self.wet_in.iter_mut().zip(&self.pending) {
            *w = s * DENOISE_SCALE;
        }
        let _voice_activity = self.state.process_frame(&mut self.wet_out, &self.wet_in);
        for (&wet, &s) in self.wet_out.iter().zip(&self.pending) {
            self.dry.push_back(s);
            let dry = self.dry.pop_front().unwrap_or(0.0);
            let mixed = wet / DENOISE_SCALE * self.mix + dry * (1.0 - self.mix);
            if self.skip > 0 {
                self.skip -= 1;
            } else if self.limit.is_none_or(|limit| self.produced < limit) {
                out.push(mixed);
                self.produced += 1;
            }
        }
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noise(n: usize, seed: u32) -> Vec<f32> {
        let mut x = seed;
        (0..n)
            .map(|_| {
                x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (x >> 8) as f32 / (1u32 << 24) as f32 - 0.5
            })
            .collect()
    }

    #[test]
    fn total_length_over_many_calls_and_finish() {
        for n in [1, 479, 480, 481, 1_000, 48_001] {
            let input = noise(n, 3);
            let mut d = Denoise::new(0.85);
            let mut out = Vec::new();
            for chunk in input.chunks(137) {
                d.process(chunk, &mut out);
            }
            d.finish(&mut out);
            assert_eq!(out.len(), n, "length {n}");
        }
    }

    #[test]
    fn silence_stays_silent() {
        let mut d = Denoise::new(0.85);
        let mut out = Vec::new();
        d.process(&vec![0.0; 10_000], &mut out);
        d.finish(&mut out);
        assert!(out.iter().all(|v| v.abs() < 1e-6));
    }

    #[test]
    fn mix_zero_gives_the_input_exactly() {
        let input = noise(5_000, 9);
        let mut d = Denoise::new(0.0);
        let mut out = Vec::new();
        d.process(&input, &mut out);
        d.finish(&mut out);
        assert_eq!(out, input);
    }
}
