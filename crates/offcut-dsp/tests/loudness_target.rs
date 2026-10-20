#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! P4 and P5 of `run_chain` (TS §34.3; v3implementation §19.4): the cleaned
//! voice reads -14 LUFS (within 0.5 LU, or 1 LU for speech and a square wave),
//! its peak is at most -1 dBFS, and the limiter's guard clamp never has work.

use std::path::PathBuf;

use offcut_dsp::compressor::Compressor;
use offcut_dsp::config::{BLOCK_SAMPLES, LOUDNESS_RETRY_LU, SAMPLE_RATE};
use offcut_dsp::denoise::Denoise;
use offcut_dsp::highpass::HighPass;
use offcut_dsp::limiter::limit;
use offcut_dsp::loudness::{apply_gain, gain_db, integrated};
use offcut_dsp::{ChainConfig, ChainOutput, Flow, run_chain};
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

const TARGET: f32 = -14.0;

fn chain(x: &[f32]) -> ChainOutput {
    run_chain(x, &ChainConfig::default(), &mut |_| Flow::Continue)
        .unwrap()
        .unwrap()
}

fn sine(freq: f32, peak_dbfs: f32, secs: f32) -> Vec<f32> {
    let peak = 10_f32.powf(peak_dbfs / 20.0);
    (0..(secs * 48_000.0) as usize)
        .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / 48_000.0).sin() * peak)
        .collect()
}

fn white(n: usize, seed: u32) -> Vec<f32> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (x >> 8) as f32 / (1u32 << 24) as f32 - 0.5
        })
        .collect()
}

/// `x` scaled to an RMS of `rms_dbfs`.
fn at_rms(mut x: Vec<f32>, rms_dbfs: f32) -> Vec<f32> {
    let rms = (x.iter().map(|&s| f64::from(s) * f64::from(s)).sum::<f64>() / x.len() as f64).sqrt();
    let k = 10_f64.powf(f64::from(rms_dbfs) / 20.0) / rms;
    x.iter_mut().for_each(|s| *s = (f64::from(*s) * k) as f32);
    x
}

/// Pink noise from white noise of a fixed seed (Paul Kellet's filter).
fn pink(n: usize) -> Vec<f32> {
    let mut b = [0.0_f32; 3];
    white(n, 77)
        .into_iter()
        .map(|w| {
            b[0] = 0.997_65 * b[0] + w * 0.099_046;
            b[1] = 0.963_00 * b[1] + w * 0.296_516_4;
            b[2] = 0.570_00 * b[2] + w * 1.052_691_3;
            b[0] + b[1] + b[2] + w * 0.184_8
        })
        .collect()
}

/// The chain's steps 2-3 (TS §34.3) from the crate's public stages: what the
/// loudness step is given.
fn before_gain(x: &[f32]) -> Vec<f32> {
    let cfg = ChainConfig::default();
    let mut work = x.to_vec();
    let (mut hpf, mut comp) = (
        HighPass::new(cfg.hpf, SAMPLE_RATE),
        Compressor::new(SAMPLE_RATE),
    );
    let (mut den, mut denoise) = (
        Vec::with_capacity(work.len()),
        Denoise::new(cfg.denoise_mix),
    );
    for block in work.chunks_mut(BLOCK_SAMPLES) {
        hpf.process(block);
        let from = den.len();
        denoise.process(block, &mut den);
        comp.process(&mut den[from..]);
    }
    let from = den.len();
    denoise.finish(&mut den);
    comp.process(&mut den[from..]);
    den
}

/// The limiter's guard clamp over the chain's two `limit` calls, by the chain's
/// own steps 4-6 (TS §34.3): `run_chain` does not return the count.
fn guard_clamps(x: &[f32]) -> usize {
    let cfg = ChainConfig::default();
    let mut work = before_gain(x);
    let measured = integrated(&work);
    let gain = gain_db(measured, &cfg);
    apply_gain(&mut work, gain);
    let mut clamped = limit(&mut work, cfg.ceiling, SAMPLE_RATE);
    let gated = measured.is_some_and(|m| m.get() >= cfg.absolute_silence.get());
    if let (true, true, Some(now)) = (gated, gain < cfg.max_gain_db, integrated(&work)) {
        let off = cfg.target.get() - now.get();
        if off.abs() > LOUDNESS_RETRY_LU {
            apply_gain(&mut work, off);
            clamped += limit(&mut work, cfg.ceiling, SAMPLE_RATE);
        }
    }
    clamped
}

/// P4 and P5 within `tolerance` LU of the target.
fn assert_on_target(x: &[f32], tolerance: f32) -> ChainOutput {
    let out = chain(x);
    assert_eq!(out.samples.len(), x.len());
    assert!(out.samples.iter().all(|s| s.is_finite()));
    assert!(out.peak.get() <= -1.0 + 1e-4, "peak {}", out.peak.get());
    assert!(
        (out.loudness.get() - TARGET).abs() <= tolerance,
        "loudness {}",
        out.loudness.get()
    );
    out
}

#[test]
fn a_quiet_sine_reaches_the_target() {
    assert_on_target(&sine(1_000.0, -30.0, 20.0), 0.5);
}

/// Pink noise at -40 dBFS RMS reads about -40 LUFS: 26 LU from the target,
/// more than `MAX_GAIN_DB`, before the denoiser takes any of it away. So this
/// is P4's exception: the gain is capped, and the output reads what the
/// loudness step was given plus 24 dB (v3changelog, Prompt 67).
#[test]
fn pink_noise_reaches_the_target() {
    let x = at_rms(pink(960_000), -40.0);
    let given = integrated(&before_gain(&x)).unwrap().get();
    assert!(
        TARGET - given > 24.0,
        "the gain is not capped: {given} LUFS"
    );
    let out = chain(&x);
    assert!(out.samples.iter().all(|s| s.is_finite()));
    assert!(out.peak.get() <= -1.0 + 1e-4, "peak {}", out.peak.get());
    assert!(
        (out.loudness.get() - (given + 24.0)).abs() <= 0.5,
        "{} from {given}",
        out.loudness.get()
    );
}

#[test]
fn modulated_noise_reaches_the_target() {
    let am: Vec<f32> = white(960_000, 31)
        .iter()
        .enumerate()
        .map(|(i, w)| {
            w * (0.5 + 0.5 * (2.0 * std::f32::consts::PI * 4.0 * i as f32 / 48_000.0).sin())
        })
        .collect();
    assert_on_target(&at_rms(am, -20.0), 0.5);
}

#[test]
fn a_full_scale_square_wave_is_held_under_the_ceiling() {
    let x: Vec<f32> = (0..480_000)
        .map(|i| if (i / 120) % 2 == 0 { 1.0 } else { -1.0 })
        .collect();
    let out = assert_on_target(&x, 1.0);
    // The loudness reported is a fresh measurement of what is returned.
    assert_eq!(Some(out.loudness), integrated(&out.samples));
}

#[test]
fn a_very_quiet_sine_gets_the_capped_gain() {
    let x = sine(1_000.0, -50.0, 20.0);
    let input = integrated(&x).unwrap().get();
    let out = chain(&x);
    assert!(out.peak.get() <= -1.0 + 1e-4);
    assert!(
        (out.loudness.get() - (input + 24.0)).abs() <= 0.5,
        "{} from {input}",
        out.loudness.get()
    );
}

/// A speech fixture: within 1 LU (PS §20.1), and within 0.5 LU unless the gain was capped.
fn speech_on_target(name: &str) {
    let x = pcm(name);
    let input = integrated(&x).unwrap().get();
    let capped = TARGET - input > 24.0;
    let out = assert_on_target(&x, 1.0);
    if !capped {
        assert!(
            (out.loudness.get() - TARGET).abs() <= 0.5,
            "{name}: {}",
            out.loudness.get()
        );
    }
}

#[test]
fn the_portrait_speech_reaches_the_target() {
    speech_on_target("speech_60s_portrait.mp4");
}

#[test]
fn the_long_pause_speech_reaches_the_target() {
    speech_on_target("speech_20s_long_pause.mp4");
}

#[test]
fn the_sparse_speech_reaches_the_target() {
    speech_on_target("speech_30s_sparse.mp4");
}

#[test]
fn digital_silence_is_left_silent() {
    let cfg = ChainConfig::default();
    let out = run_chain(&vec![0.0; 480_000], &cfg, &mut |_| Flow::Continue);
    let out = out.expect("silence is not an error").unwrap();
    assert!(out.samples.iter().all(|&s| s == 0.0));
    assert_eq!(out.loudness, cfg.absolute_silence);
}

#[test]
fn the_guard_clamp_touches_nothing() {
    let square: Vec<f32> = (0..480_000)
        .map(|i| if (i / 120) % 2 == 0 { 1.0 } else { -1.0 })
        .collect();
    let cases: Vec<(&str, Vec<f32>)> = vec![
        ("sine -30", sine(1_000.0, -30.0, 20.0)),
        ("pink -40", at_rms(pink(960_000), -40.0)),
        ("square", square),
        ("sine -50", sine(1_000.0, -50.0, 20.0)),
        ("portrait", pcm("speech_60s_portrait.mp4")),
        ("long pause", pcm("speech_20s_long_pause.mp4")),
        ("sparse", pcm("speech_30s_sparse.mp4")),
        ("silence", vec![0.0; 480_000]),
    ];
    for (name, x) in cases {
        assert_eq!(guard_clamps(&x), 0, "{name}");
    }
}
