#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! P1, P2, P3 of `run_chain` (TS §34.3; INV-10): the cleaned voice is as long
//! as the input, at the same times, and a silent span stays where it was. The
//! test names are TS §34.3's. This is also the test that fixes
//! `DENOISE_DELAY_SAMPLES` (v3implementation §19.6).

use std::path::PathBuf;

use offcut_dsp::config::{ALIGN_TOLERANCE, DENOISE_DELAY_SAMPLES};
use offcut_dsp::denoise::Denoise;
use offcut_dsp::{ChainConfig, ChainOutput, Flow, run_chain};
use proptest::prelude::*;
use sha2::{Digest, Sha256};

const STALE: &str = "the generated fixtures are stale or missing: run `pnpm fixtures`";

/// A generated fixture's bytes, after checking that `fixtures/.gen/` was built
/// from the manifest of this checkout (v3implementation §6.4).
fn fixture(name: &str) -> Vec<u8> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = std::env::var_os("OFFCUT_FIXTURES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("fixtures/.gen"));
    let stamp = std::fs::read_to_string(dir.join("STAMP")).expect(STALE);
    let manifest = std::fs::read(root.join("fixtures/manifest.json")).expect(STALE);
    let sha: String = Sha256::digest(&manifest)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert!(
        stamp.contains(&format!("\"manifest_sha256\": \"{sha}\"")),
        "{STALE}"
    );
    std::fs::read(dir.join(name)).expect(STALE)
}

/// A speech fixture's mono 48 kHz PCM (`<name>.pcm48.f32`, little-endian `f32`).
fn pcm(name: &str) -> Vec<f32> {
    let bytes = fixture(&format!("speech/{name}.pcm48.f32"));
    let (frames, _) = bytes.as_chunks::<4>();
    frames.iter().map(|b| f32::from_le_bytes(*b)).collect()
}

const SPEECH: [&str; 5] = [
    "speech_60s_portrait.mp4",
    "speech_60s_landscape.mp4",
    "speech_20s_noevents.mp4",
    "speech_20s_long_pause.mp4",
    "speech_30s_sparse.mp4",
];

fn chain(x: &[f32]) -> ChainOutput {
    run_chain(x, &ChainConfig::default(), &mut |_| Flow::Continue)
        .unwrap()
        .unwrap()
}

/// Deterministic noise in [-0.5, 0.5) from a fixed-seed LCG.
fn noise(n: usize, seed: u32) -> Vec<f32> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (x >> 8) as f32 / (1u32 << 24) as f32 - 0.5
        })
        .collect()
}

/// Noise shaped like speech: its level rises and falls four times a second.
fn speechlike(n: usize, seed: u32) -> Vec<f32> {
    noise(n, seed)
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v * 0.4 * (0.6 + 0.4 * (2.0 * std::f32::consts::PI * 4.0 * i as f32 / 48_000.0).sin())
        })
        .collect()
}

fn rms_db(x: &[f32]) -> f64 {
    let ms = x.iter().map(|&s| f64::from(s) * f64::from(s)).sum::<f64>() / x.len() as f64;
    10.0 * ms.max(1e-30).log10()
}

fn secs(s: f64) -> usize {
    (s * 48_000.0) as usize
}

#[test]
fn length_equals_input_fixtures() {
    for name in SPEECH {
        let input = pcm(name);
        assert_eq!(chain(&input).samples.len(), input.len(), "{name}");
    }
}

/// Lengths from 1 to 300,000, half of the cases on the block (48,000) and
/// frame (480) edges, where a bookkeeping error would show first.
fn length() -> impl Strategy<Value = usize> {
    prop_oneof![
        1usize..=300_000,
        prop::sample::select(vec![
            1, 2, 479, 480, 481, 959, 961, 47_999, 48_000, 48_001, 96_001
        ]),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]

    #[test]
    fn prop_length_equals_input(n in length(), seed in any::<u32>()) {
        let input = noise(n, seed);
        let out = chain(&input);
        prop_assert_eq!(out.samples.len(), n);
        prop_assert!(out.samples.iter().all(|s| s.is_finite()));
    }
}

#[test]
fn speech_silence_speech_stays_20s() {
    let mut input = speechlike(secs(20.0), 11);
    input[secs(5.0)..secs(10.0)].fill(0.0);
    let out = chain(&input).samples;
    assert_eq!(out.len(), 960_000);
    let quiet = rms_db(&out[secs(5.1)..secs(9.9)]);
    let before = rms_db(&out[..secs(5.0)]);
    let after = rms_db(&out[secs(10.0)..]);
    assert!(
        quiet <= before - 40.0 && quiet <= after - 40.0,
        "{quiet} vs {before}, {after}"
    );
}

/// Where the largest sample of `x` lies within `radius` of each position.
fn offsets(x: &[f32], at: &[usize], radius: usize) -> Vec<i64> {
    at.iter()
        .map(|&p| {
            let (lo, hi) = (p.saturating_sub(radius), (p + radius).min(x.len()));
            let best = (lo..hi).fold(lo, |b, i| if x[i].abs() > x[b].abs() { i } else { b });
            best as i64 - p as i64
        })
        .collect()
}

#[test]
fn impulses_stay_aligned() {
    // One impulse every 0.5 s on a noise floor of -40 dBFS RMS.
    let n = secs(10.0);
    let floor = 10_f32.powf(-40.0 / 20.0) / 0.2887;
    let mut input: Vec<f32> = noise(n, 5).iter().map(|v| v * floor).collect();
    let at: Vec<usize> = (1..20).map(|k| k * secs(0.5)).collect();
    for &p in &at {
        input[p] = 0.5;
    }

    // The denoiser alone, all wet: its own offset after compensation.
    let mut wet = Vec::new();
    let mut d = Denoise::new(1.0);
    d.process(&input, &mut wet);
    d.finish(&mut wet);
    let wet_offsets = offsets(&wet, &at, 2_400);
    let out = chain(&input).samples;
    let chain_offsets = offsets(&out, &at, 2_400);
    println!(
        "DENOISE_DELAY_SAMPLES = {DENOISE_DELAY_SAMPLES}; offsets, denoiser alone (mix 1.0): {wet_offsets:?}"
    );
    println!("offsets, whole chain: {chain_offsets:?}");
    for (name, offs) in [("denoiser alone", &wet_offsets), ("chain", &chain_offsets)] {
        assert!(
            offs.iter()
                .all(|o| o.unsigned_abs() as usize <= ALIGN_TOLERANCE),
            "{name}: an impulse moved more than {ALIGN_TOLERANCE} samples: {offs:?}"
        );
    }
}

#[test]
fn mostly_silent_clip_keeps_length() {
    // 60 s: silence, then 3 s of a 440 Hz tone from 30.0 s, then silence (5% tone).
    let mut input = vec![0.0_f32; secs(60.0)];
    for (i, s) in input[secs(30.0)..secs(33.0)].iter_mut().enumerate() {
        *s = (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48_000.0).sin() * 0.3;
    }
    let out = chain(&input).samples;
    assert_eq!(out.len(), input.len());
    let peak = out.iter().fold(0.0_f32, |m, v| m.max(v.abs()));
    let onset = out.iter().position(|v| v.abs() > 0.1 * peak).unwrap();
    assert!(
        onset.abs_diff(secs(30.0)) <= ALIGN_TOLERANCE,
        "tone starts at {onset}"
    );
}

#[test]
fn digital_silence_keeps_length() {
    let input = vec![0.0_f32; 4_320_000];
    let out = run_chain(&input, &ChainConfig::default(), &mut |_| Flow::Continue);
    let out = out.expect("silence is not an error").unwrap();
    assert_eq!(out.samples.len(), 4_320_000);
    assert!(out.samples.iter().all(|&s| s == 0.0));
}

#[test]
fn long_pause_fixture_keeps_its_pause() {
    let input = pcm("speech_20s_long_pause.mp4");
    let out = chain(&input).samples;
    assert_eq!(out.len(), input.len());
    let pause = rms_db(&out[secs(5.2)..secs(9.8)]);
    let speech = rms_db(&out[secs(0.5)..secs(4.5)]);
    assert!(pause < -50.0, "the pause is at {pause} dBFS");
    assert!(speech > -40.0, "the speech is at {speech} dBFS");
}
