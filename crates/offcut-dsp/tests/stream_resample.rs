#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! `StreamResampler` fed in pieces equals `resample_mono` over the whole input,
//! sample for sample (v6.1implementation §7.1, D-401).

use offcut_dsp::resample::{StreamResampler, resample_mono};
use offcut_types::Hz;

/// A seeded generator: the same pieces and the same noise on every run.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
    fn noise(&mut self) -> f32 {
        (self.next() % 20_001) as f32 / 10_000.0 - 1.0
    }
}

fn noise(len: usize, seed: u64) -> Vec<f32> {
    let mut g = Lcg(seed);
    (0..len).map(|_| g.noise()).collect()
}

/// `input` through a `StreamResampler`, in pieces of 1 to `max_piece` samples.
fn streamed(input: &[f32], from: u32, to: u32, max_piece: u64, seed: u64) -> Vec<f32> {
    let mut g = Lcg(seed);
    let mut r = StreamResampler::new(Hz::new(from), Hz::new(to)).unwrap();
    let mut out = Vec::with_capacity(input.len());
    let mut at = 0;
    while at < input.len() {
        let n = (1 + g.next() % max_piece) as usize;
        let end = (at + n).min(input.len());
        out.extend(r.push(&input[at..end]));
        at = end;
    }
    out.extend(r.finish(input.len()));
    out
}

fn bits(samples: &[f32]) -> Vec<u32> {
    samples.iter().map(|s| s.to_bits()).collect()
}

fn assert_equal(input: &[f32], from: u32, to: u32, max_piece: u64, seed: u64) {
    let whole = resample_mono(input, Hz::new(from), Hz::new(to));
    let pieces = streamed(input, from, to, max_piece, seed);
    assert_eq!(
        pieces.len(),
        whole.len(),
        "{} samples {from} -> {to}",
        input.len()
    );
    assert!(
        bits(&pieces) == bits(&whole),
        "{} samples {from} -> {to}: the streamed output differs",
        input.len()
    );
}

#[test]
fn two_minutes_at_48_and_44_1_khz_in_random_pieces_equal_resample_mono() {
    // Two minutes, not the plan's seven: seven take 128 s in a debug build
    // (P173, read on D1) and prove nothing more, every chunk being alike.
    // Pieces of up to 10 s, as `decodeAudioRegion` hands them (D-402), and smaller.
    for (from, seed) in [(48_000u32, 7u64), (44_100, 11)] {
        let input = noise(2 * 60 * from as usize, seed);
        assert_equal(&input, from, 16_000, u64::from(from) * 10, seed + 1);
        assert_equal(&input, from, 16_000, 3_000, seed + 2);
    }
}

#[test]
fn lengths_around_one_chunk_equal_resample_mono() {
    for len in [
        1usize, 2, 100, 1_023, 1_024, 1_025, 2_047, 2_048, 2_049, 3_073, 48_000, 48_001,
    ] {
        let input = noise(len, len as u64);
        for (from, to) in [(48_000u32, 16_000u32), (44_100, 16_000), (44_100, 48_000)] {
            for max_piece in [1u64, 7, 1_024, 5_000] {
                assert_equal(&input, from, to, max_piece, len as u64 + max_piece);
            }
        }
    }
}

#[test]
fn equal_rates_pass_through_and_a_zero_rate_is_refused() {
    let input = noise(10_000, 3);
    assert_eq!(
        bits(&streamed(&input, 16_000, 16_000, 999, 4)),
        bits(&input)
    );
    assert!(StreamResampler::new(Hz::new(0), Hz::new(16_000)).is_none());
    assert!(StreamResampler::new(Hz::new(48_000), Hz::new(0)).is_none());
    let r = StreamResampler::new(Hz::new(48_000), Hz::new(16_000)).unwrap();
    assert!(r.finish(0).is_empty());
}

#[test]
fn a_rate_change_mid_stream_is_two_resamplers_each_equal_to_resample_mono() {
    // D-402: a decoded chunk at another rate rebuilds the resampler; each part
    // then equals `resample_mono` of that part.
    let a = noise(5 * 44_100, 21);
    let b = noise(5 * 48_000, 22);
    let mut out = streamed(&a, 44_100, 16_000, 4_410, 23);
    out.extend(streamed(&b, 48_000, 16_000, 4_800, 24));
    let mut whole = resample_mono(&a, Hz::new(44_100), Hz::new(16_000));
    whole.extend(resample_mono(&b, Hz::new(48_000), Hz::new(16_000)));
    assert_eq!(bits(&out), bits(&whole));
}
