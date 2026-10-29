#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Program audio on the identity map (TS §40.6, v4implementation §10): on the
//! cleaned voice of two derived recordings, `assemble` gives every sample
//! back and reads the chain's loudness; it refuses any other map and any
//! music; and `silence_profile` reads 10 ms windows.

use std::path::PathBuf;

use offcut_dsp::silence::FLOOR_DB;
use offcut_dsp::{
    ChainConfig, ChainOutput, DspError, Flow, MusicInput, ProgramConfig, assemble, run_chain,
    silence_profile,
};
use offcut_types::{DurMs, Lufs, TimeMap};
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

fn chain(x: &[f32]) -> ChainOutput {
    run_chain(x, &ChainConfig::default(), &mut |_| Flow::Continue)
        .unwrap()
        .unwrap()
}

/// The identity map of `samples` at 48 kHz, as the page builds one.
fn identity_of(samples: &[f32]) -> TimeMap {
    TimeMap::identity(DurMs::new(u32::try_from(samples.len() / 48).unwrap()))
}

#[test]
fn on_the_identity_map_the_program_is_the_cleaned_voice() {
    for name in ["speech_60s_portrait.mp4", "speech_20s_long_pause.mp4"] {
        let out = chain(&pcm(name));
        let program = assemble(
            &out.samples,
            &identity_of(&out.samples),
            &[],
            None,
            &ProgramConfig::default(),
        )
        .unwrap();
        assert_eq!(program.samples.len(), out.samples.len(), "{name}");
        // Sample for sample, bit for bit.
        let same = program
            .samples
            .iter()
            .zip(&out.samples)
            .all(|(a, b)| a.to_bits() == b.to_bits());
        assert!(same, "{name}: a sample changed");
        let lu = (program.loudness.get() - out.loudness.get()).abs();
        assert!(
            lu <= 0.01,
            "{name}: {} vs {} LUFS",
            program.loudness.get(),
            out.loudness.get()
        );
    }
}

#[test]
fn a_map_that_is_not_the_identity_and_music_are_refused() {
    let voice = chain(&pcm("speech_20s_long_pause.mp4")).samples;
    let cfg = ProgramConfig::default();
    // A second out of the middle taken away: what V6.2 will make. V4 builds
    // no such map outside a test (D-164).
    let cut: TimeMap = serde_json::from_str(
        r#"{ "segments": [
            { "kind": "source", "src": { "start": 0, "end": 9000 } },
            { "kind": "source", "src": { "start": 10000, "end": 20000 } }
        ] }"#,
    )
    .unwrap();
    assert!(!cut.is_identity());
    assert_eq!(
        assemble(&voice, &cut, &[], None, &cfg),
        Err(DspError::Unsupported)
    );
    // A hold of the frame at 5 s is not the identity either.
    let held: TimeMap = serde_json::from_str(
        r#"{ "segments": [
            { "kind": "source", "src": { "start": 0, "end": 20000 } },
            { "kind": "hold", "at": 5000, "dur": 500 }
        ] }"#,
    )
    .unwrap();
    assert_eq!(
        assemble(&voice, &held, &[], None, &cfg),
        Err(DspError::Unsupported)
    );
    let music = MusicInput {
        pcm48: &voice,
        loudness: Lufs::new(-20.0).unwrap(),
    };
    assert_eq!(
        assemble(&voice, &identity_of(&voice), &[], Some(music), &cfg),
        Err(DspError::Unsupported)
    );
}

#[test]
fn silence_reads_the_floor_and_a_full_scale_sine_minus_3_db() {
    let zeros = silence_profile(&[0.0; 48_000]);
    assert_eq!(zeros.rms_db_10ms, vec![FLOOR_DB; 100]);
    // 1 kHz at full scale: ten whole periods in every window, RMS 1/√2.
    let sine: Vec<f32> = (0..48_000)
        .map(|i| (2.0 * std::f64::consts::PI * 1_000.0 * f64::from(i) / 48_000.0).sin() as f32)
        .collect();
    let profile = silence_profile(&sine);
    assert_eq!(profile.rms_db_10ms.len(), 100);
    for db in profile.rms_db_10ms {
        assert!((db - -3.0103).abs() < 0.01, "{db}");
    }
}
