#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! P6, P7, P8 of `run_chain` and its two errors (TS §34.3; v3implementation
//! §19.5): no click where one 1 s block meets the next, cancellation between
//! blocks, the same bits for the same input.

use std::path::PathBuf;

use offcut_dsp::config::{BLOCK_SAMPLES, CLICK_FACTOR, CLICK_RMS_WINDOW};
use offcut_dsp::{ChainConfig, DspError, Flow, run_chain};
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

fn chain(x: &[f32], cfg: &ChainConfig) -> Vec<f32> {
    run_chain(x, cfg, &mut |_| Flow::Continue)
        .unwrap()
        .unwrap()
        .samples
}

fn noise(n: usize, seed: u32) -> Vec<f32> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (x >> 8) as f32 / (1u32 << 24) as f32 - 0.5
        })
        .collect()
}

/// At every internal block boundary `b`: the step from `out[b-1]` to `out[b]`
/// is at most `CLICK_FACTOR` times the RMS of `out[b-480 .. b+480]` (P6).
fn assert_no_clicks(out: &[f32]) -> usize {
    let mut checked = 0;
    for b in (BLOCK_SAMPLES..out.len()).step_by(BLOCK_SAMPLES) {
        if b + CLICK_RMS_WINDOW > out.len() {
            break;
        }
        let window = &out[b - CLICK_RMS_WINDOW..b + CLICK_RMS_WINDOW];
        let rms = (window
            .iter()
            .map(|&s| f64::from(s) * f64::from(s))
            .sum::<f64>()
            / window.len() as f64)
            .sqrt();
        let step = f64::from((out[b] - out[b - 1]).abs());
        assert!(
            step <= f64::from(CLICK_FACTOR) * rms,
            "boundary {b}: step {step} against RMS {rms}"
        );
        checked += 1;
    }
    checked
}

#[test]
fn a_sine_has_no_click_at_a_boundary() {
    let peak = 10_f32.powf(-20.0 / 20.0);
    let x: Vec<f32> = (0..264_000)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48_000.0).sin() * peak)
        .collect();
    assert_eq!(assert_no_clicks(&chain(&x, &ChainConfig::default())), 5);
}

#[test]
fn noise_has_no_click_at_a_boundary() {
    let scale = 10_f32.powf(-25.0 / 20.0) / 0.2887;
    let x: Vec<f32> = noise(153_600, 21).iter().map(|v| v * scale).collect();
    assert_eq!(assert_no_clicks(&chain(&x, &ChainConfig::default())), 3);
}

#[test]
fn speech_has_no_click_at_any_of_its_59_boundaries() {
    let x = pcm("speech_60s_portrait.mp4");
    assert_eq!(assert_no_clicks(&chain(&x, &ChainConfig::default())), 59);
}

#[test]
fn empty_input_is_an_error() {
    assert_eq!(
        run_chain(&[], &ChainConfig::default(), &mut |_| Flow::Continue),
        Err(DspError::Empty)
    );
}

#[test]
fn a_non_finite_sample_is_an_error() {
    for bad in [f32::NAN, f32::INFINITY] {
        let mut x = vec![0.1_f32; 5_000];
        x[1_000] = bad;
        assert_eq!(
            run_chain(&x, &ChainConfig::default(), &mut |_| Flow::Continue),
            Err(DspError::NonFinite)
        );
    }
}

#[test]
fn cancel_on_the_third_call_stops_there() {
    let mut calls = 0;
    let out = run_chain(&noise(300_000, 4), &ChainConfig::default(), &mut |_| {
        calls += 1;
        if calls == 3 {
            Flow::Cancel
        } else {
            Flow::Continue
        }
    });
    assert_eq!(out, Ok(None));
    assert_eq!(calls, 3);
}

#[test]
fn the_same_input_gives_the_same_bits() {
    let x: Vec<f32> = noise(150_000, 8).iter().map(|v| v * 0.2).collect();
    for cfg in [
        ChainConfig::default(),
        ChainConfig {
            denoise: false,
            ..ChainConfig::default()
        },
    ] {
        let (a, b) = (chain(&x, &cfg), chain(&x, &cfg));
        let bits = |v: &[f32]| v.iter().map(|s| s.to_bits()).collect::<Vec<_>>();
        assert_eq!(bits(&a), bits(&b), "denoise {}", cfg.denoise);
    }
}

/// The one clock in this crate's code: a timing read by hand (TE-13), ignored
/// by default, asserting nothing. The crate itself reads no clock (TS §7).
#[test]
#[ignore = "a timing, read by hand: cargo test -p offcut-dsp --release --test chain_no_clicks -- --ignored --nocapture"]
#[allow(clippy::disallowed_methods)]
fn chain_timing() {
    let x = pcm("speech_60s_portrait.mp4");
    let start = std::time::Instant::now();
    let out = chain(&x, &ChainConfig::default());
    let took = start.elapsed();
    println!(
        "run_chain: {} samples ({:.1} s of audio) in {:.3} s, {:.0} samples per second",
        out.len(),
        x.len() as f64 / 48_000.0,
        took.as_secs_f64(),
        x.len() as f64 / took.as_secs_f64()
    );
}
