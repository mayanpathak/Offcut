#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! `measure_prosody` (v3implementation §7.9, §19.7): per-word energy and pitch
//! z-scores within each sentence, on synthetic words (sawtooth bursts of a
//! known pitch and level, separated by silence) and on the portrait speech.

use std::path::PathBuf;

use offcut_dsp::measure_prosody;
use offcut_types::{Confidence, Sentence, TimeMs, Word, WordIdx, WordProsody, WordRange};
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

fn word(text: &str, start_ms: u32, end_ms: u32) -> Word {
    Word {
        text: text.to_owned(),
        start_ms: TimeMs::new(start_ms),
        end_ms: TimeMs::new(end_ms),
        confidence: Confidence::new(1.0).unwrap(),
    }
}

fn sentence(from: u32, to: u32, words: &[Word]) -> Sentence {
    Sentence {
        words: WordRange {
            start: WordIdx::new(from),
            end: WordIdx::new(to),
        },
        start_ms: words[from as usize].start_ms,
        end_ms: words[to as usize - 1].end_ms,
    }
}

/// `n` words of 300 ms with 100 ms of silence between them, one sentence.
fn layout(n: usize) -> (Vec<Word>, Vec<Sentence>) {
    let words: Vec<Word> = (0..n as u32)
        .map(|i| word(&format!("w{i}"), 100 + i * 400, 400 + i * 400))
        .collect();
    let s = if n == 0 {
        vec![]
    } else {
        vec![sentence(0, n as u32, &words)]
    };
    (words, s)
}

/// A sawtooth of `f0` Hz and peak `peak_dbfs` under each word; silence elsewhere.
fn render(words: &[Word], tones: &[(f32, f32)]) -> Vec<f32> {
    let end = words
        .last()
        .map_or(48_000, |w| w.end_ms.get() as usize * 48 + 4_800);
    let mut x = vec![0.0_f32; end];
    for (w, &(f0, peak_dbfs)) in words.iter().zip(tones) {
        let peak = 10_f32.powf(peak_dbfs / 20.0);
        let (a, b) = (
            w.start_ms.get() as usize * 48,
            (w.end_ms.get() as usize * 48).min(end),
        );
        for (i, s) in x[a..b].iter_mut().enumerate() {
            *s = (2.0 * (i as f32 * f0 / 48_000.0).fract() - 1.0) * peak;
        }
    }
    x
}

fn energy(p: &[WordProsody]) -> Vec<f32> {
    p.iter().map(|w| w.energy_z).collect()
}

fn pitch(p: &[WordProsody]) -> Vec<f32> {
    p.iter().map(|w| w.pitch_z).collect()
}

#[test]
fn a_louder_fourth_word_stands_out_in_energy() {
    let (words, sentences) = layout(5);
    let x = render(
        &words,
        &[
            (120.0, -20.0),
            (120.0, -20.0),
            (120.0, -20.0),
            (120.0, -8.0),
            (120.0, -20.0),
        ],
    );
    let e = energy(&measure_prosody(&x, &words, &sentences).per_word);
    assert!(e[3] >= 1.5, "{e:?}");
    assert!(
        e.iter().enumerate().all(|(i, &v)| i == 3 || v <= 0.5),
        "{e:?}"
    );
}

#[test]
fn a_higher_third_word_stands_out_in_pitch() {
    let (words, sentences) = layout(5);
    let x = render(
        &words,
        &[
            (120.0, -20.0),
            (120.0, -20.0),
            (200.0, -20.0),
            (120.0, -20.0),
            (120.0, -20.0),
        ],
    );
    let z = pitch(&measure_prosody(&x, &words, &sentences).per_word);
    assert!(z[2] >= 1.5 && z.iter().all(|&v| v <= z[2]), "{z:?}");
}

#[test]
fn an_unvoiced_word_has_no_pitch_z() {
    let (words, sentences) = layout(5);
    let mut x = render(
        &words,
        &[
            (110.0, -20.0),
            (130.0, -20.0),
            (0.0, -200.0),
            (150.0, -20.0),
            (170.0, -20.0),
        ],
    );
    let (a, b) = (
        words[2].start_ms.get() as usize * 48,
        words[2].end_ms.get() as usize * 48,
    );
    let mut seed = 12_345_u32;
    for s in &mut x[a..b] {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        *s = ((seed >> 8) as f32 / (1u32 << 24) as f32 - 0.5) * 0.2;
    }
    let z = pitch(&measure_prosody(&x, &words, &sentences).per_word);
    assert_eq!(z[2], 0.0, "{z:?}");
    assert!(
        z.iter().any(|&v| v != 0.0),
        "the voiced words have pitch z-scores: {z:?}"
    );
}

#[test]
fn a_sentence_of_two_words_is_all_zero() {
    let (words, sentences) = layout(2);
    let x = render(&words, &[(120.0, -20.0), (240.0, -6.0)]);
    for w in measure_prosody(&x, &words, &sentences).per_word {
        assert_eq!((w.energy_z, w.pitch_z), (0.0, 0.0));
    }
}

#[test]
fn a_word_past_the_end_gets_the_energy_floor() {
    let (mut words, _) = layout(4);
    let x = render(&words, &[(120.0, -20.0); 4]);
    let end = (x.len() / 48) as u32;
    words.push(word("late", end + 1_000, end + 1_300));
    let sentences = vec![sentence(0, 5, &words)];
    let e = energy(&measure_prosody(&x, &words, &sentences).per_word);
    assert!(e.iter().all(|v| v.is_finite()));
    assert!(
        e[4] < 0.0 && e.iter().all(|&v| v >= e[4]),
        "the late word is the quietest: {e:?}"
    );
}

#[test]
fn one_entry_per_word() {
    for n in [0, 1, 150] {
        let (words, sentences) = layout(n);
        let x = render(&words, &vec![(120.0, -20.0); n]);
        assert_eq!(measure_prosody(&x, &words, &sentences).per_word.len(), n);
    }
}

#[test]
fn one_and_two_semitones_up_rise_in_pitch_z() {
    for f in [80.0_f32, 150.0, 300.0] {
        let (words, sentences) = layout(3);
        let x = render(&words, &[(f, -20.0), (f * 1.06, -20.0), (f * 1.12, -20.0)]);
        let z = pitch(&measure_prosody(&x, &words, &sentences).per_word);
        assert!(z[0] < z[1] && z[1] < z[2], "{f} Hz: {z:?}");
    }
}

/// Sentences 3 and 4 of "The V2 reference clip as recognized"
/// (`fixtures/speech/README.md`), in the portrait clip's time (the reference
/// minus its first 960 ms). The README gives each sentence's start and end, not
/// each word's: the words share their sentence's time by their letters. A
/// printed reading, asserted on nothing.
#[test]
fn the_brutal_reading_of_the_portrait_speech() {
    let x = pcm("speech_60s_portrait.mp4");
    let spoken: [(&str, u32, u32); 2] = [
        (
            "I had spent 3000 hours editing by hand.",
            10_580 - 960,
            14_100 - 960,
        ),
        (
            "This is literally brutal way to make anything.",
            14_260 - 960,
            17_960 - 960,
        ),
    ];
    let mut words = Vec::new();
    let mut sentences = Vec::new();
    for (text, start, end) in spoken {
        let pieces: Vec<&str> = text.split_whitespace().collect();
        let letters: u32 = pieces.iter().map(|p| p.len() as u32).sum();
        let first = words.len() as u32;
        let mut at = start;
        for p in &pieces {
            let len = (end - start) * p.len() as u32 / letters;
            words.push(word(p, at, at + len));
            at += len;
        }
        sentences.push(sentence(first, words.len() as u32, &words));
    }
    let p = measure_prosody(&x, &words, &sentences).per_word;
    for (w, z) in words.iter().zip(&p) {
        println!(
            "{:>12} {:>6}-{:<6} energy_z {:+.2}  pitch_z {:+.2}",
            w.text,
            w.start_ms.get(),
            w.end_ms.get(),
            z.energy_z,
            z.pitch_z
        );
    }
}

#[test]
fn the_same_inputs_give_the_same_prosody() {
    let (words, sentences) = layout(6);
    let x = render(
        &words,
        &[
            (100.0, -20.0),
            (140.0, -14.0),
            (180.0, -26.0),
            (220.0, -20.0),
            (110.0, -10.0),
            (90.0, -30.0),
        ],
    );
    assert_eq!(
        measure_prosody(&x, &words, &sentences),
        measure_prosody(&x, &words, &sentences)
    );
}

/// The one clock of this file: a budget read on D1, asserted in release only
/// (TS §8: under 1 s for the reference clip; 2 s here for 60 s and 150 words).
#[test]
#[allow(clippy::disallowed_methods)]
fn the_budget_for_60_s_and_150_words() {
    let words: Vec<Word> = (0..150_u32)
        .map(|i| word(&format!("w{i}"), i * 400, i * 400 + 320))
        .collect();
    let sentences: Vec<Sentence> = (0..15_u32)
        .map(|s| sentence(s * 10, s * 10 + 10, &words))
        .collect();
    let tones: Vec<(f32, f32)> = (0..150)
        .map(|i| (100.0 + (i % 7) as f32 * 15.0, -20.0))
        .collect();
    let mut x = render(&words, &tones);
    x.resize(2_880_000, 0.0);
    let start = std::time::Instant::now();
    let p = measure_prosody(&x, &words, &sentences);
    let took = start.elapsed().as_secs_f64();
    println!("measure_prosody: 60 s, 150 words in {took:.3} s");
    assert_eq!(p.per_word.len(), 150);
    #[cfg(not(debug_assertions))]
    assert!(took < 2.0, "{took} s");
}
