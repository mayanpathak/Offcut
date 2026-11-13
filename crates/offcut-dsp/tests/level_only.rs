#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The levelled voice (v6.2implementation D-487, §13.1): `level_only` keeps
//! the length, reaches the loudness target, holds the ceiling, and does
//! nothing else (one gain where the limiter does not act); `run_chain` on the
//! same input is unchanged by it. On two derived recordings and on synthetic
//! signals.

use std::path::PathBuf;

use offcut_dsp::loudness::{integrated, sample_peak};
use offcut_dsp::{ChainConfig, ChainOutput, Flow, level_only, run_chain};
use sha2::{Digest, Sha256};

const STALE: &str = "the generated fixtures are stale or missing: run `pnpm fixtures`";
const SPEECH: [&str; 2] = ["speech_60s_portrait.mp4", "speech_20s_long_pause.mp4"];

/// A generated fixture's bytes, after checking that `fixtures/.gen/` was built
/// from the manifest of this checkout (as `program_identity.rs` reads them).
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

fn level(x: &[f32]) -> ChainOutput {
    level_only(x, &ChainConfig::default()).unwrap()
}

fn chain(x: &[f32]) -> ChainOutput {
    run_chain(x, &ChainConfig::default(), &mut |_| Flow::Continue)
        .unwrap()
        .unwrap()
}

fn sine(n: usize, hz: f32, peak: f32) -> Vec<f32> {
    (0..n)
        .map(|i| (2.0 * std::f32::consts::PI * hz * i as f32 / 48_000.0).sin() * peak)
        .collect()
}

fn same_bits(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
}

#[test]
fn the_length_is_the_inputs() {
    for n in [1, 479, 48_000, 48_001, 150_000] {
        assert_eq!(level(&sine(n, 440.0, 0.1)).samples.len(), n, "{n}");
    }
    for name in SPEECH {
        let x = pcm(name);
        assert_eq!(level(&x).samples.len(), x.len(), "{name}");
    }
}

#[test]
fn the_loudness_is_the_target_as_the_chains_meter_reads_it() {
    let cfg = ChainConfig::default();
    for name in SPEECH {
        let out = level(&pcm(name));
        let read = integrated(&out.samples).unwrap();
        assert_eq!(out.loudness, read, "{name}: the reported reading");
        assert!(
            (read.get() - cfg.target.get()).abs() <= 0.5,
            "{name}: {} LUFS",
            read.get()
        );
    }
    // A quiet sine, under the target, and a loud one, over it.
    for peak in [10_f32.powf(-30.0 / 20.0), 0.9] {
        let out = level(&sine(480_000, 1_000.0, peak));
        assert!(
            (out.loudness.get() - cfg.target.get()).abs() <= 0.5,
            "peak {peak}: {} LUFS",
            out.loudness.get()
        );
    }
    // One too quiet for the gain's cap (+24 dB, as the chain's): a 1 kHz sine
    // at -40 dBFS reads -40 LUFS, so it rises to -16, not to the target.
    let capped = level(&sine(480_000, 1_000.0, 10_f32.powf(-40.0 / 20.0)));
    let want = -40.0 + cfg.max_gain_db;
    assert!(
        (capped.loudness.get() - want).abs() <= 0.1,
        "{}",
        capped.loudness.get()
    );
}

#[test]
fn no_sample_passes_the_ceiling() {
    let ceiling = ChainConfig::default().ceiling.get();
    for name in SPEECH {
        let out = level(&pcm(name));
        assert!(
            out.peak.get() <= ceiling + 1e-3,
            "{name}: {}",
            out.peak.get()
        );
        assert_eq!(out.peak, sample_peak(&out.samples), "{name}");
    }
    // A full-scale sine: limited to the ceiling, not under it by much.
    let out = level(&sine(480_000, 1_000.0, 1.0));
    assert!(out.peak.get() <= ceiling + 1e-3, "{}", out.peak.get());
}

#[test]
fn where_the_limiter_does_not_act_the_voice_is_the_input_times_one_gain() {
    // 440 Hz at -30 dBFS peak: brought up about 20 dB, still under the
    // ceiling. No high-pass, denoise or compressor: one factor for every
    // sample, so the sound is the recording's own.
    let x = sine(480_000, 440.0, 10_f32.powf(-30.0 / 20.0));
    let out = level(&x);
    let (i, big) = x
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
        .unwrap();
    let factor = out.samples[i] / big;
    assert!(factor > 5.0, "the level rose: {factor}");
    for (n, (y, x)) in out.samples.iter().zip(&x).enumerate() {
        assert!(
            (y - x * factor).abs() <= 1e-5,
            "sample {n}: {y} vs {}",
            x * factor
        );
    }
    // The chain's own voice of the same input differs: it filters and
    // compresses.
    assert!(!same_bits(&chain(&x).samples, &out.samples));
}

#[test]
fn run_chain_on_the_same_input_is_unchanged_and_the_input_is_untouched() {
    for name in SPEECH {
        let x = pcm(name);
        let copy = x.clone();
        let before = chain(&x);
        let levelled = level(&x);
        let after = chain(&x);
        assert!(same_bits(&x, &copy), "{name}: the input changed");
        assert!(same_bits(&before.samples, &after.samples), "{name}");
        assert_eq!((before.loudness, before.peak), (after.loudness, after.peak));
        assert!(!same_bits(&before.samples, &levelled.samples), "{name}");
    }
}
