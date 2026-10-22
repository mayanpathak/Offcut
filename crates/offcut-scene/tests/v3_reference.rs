//! The identity reference (v4implementation D-116, D-117). Temporary: written
//! in V4 Prompt 93 by V3's scene code (the files under
//! `tests/snapshots/v3_reference/`, frozen at `09b1085`), compared here
//! through the V4 contract, deleted with its six JSON files in Prompt 101.
//!
//! The comparison form (G 6.1, V4 Prompt 98): the same six inputs, built as
//! rules plan → `validate` → `build_scene` on `TimeMap::identity`,
//! `Theme::default_for(Clean)` and `FontSet::embedded()`. Every 7th frame's
//! display list is serialized, normalized (the leading `Video` command
//! removed, every `FontKey(n)` read as the name of V3's font of index `n`) and
//! compared value for value, `f32` by equality, with the file. Nothing is
//! written. Passed at S6 (V4 Prompt 100).
//!
//! `busy_1080.json` and `busy_540.json` were written again in Prompt 100 by
//! V3's scene code (`09b1085`), with `busy()`'s first two NumberReveals on the
//! spans of the words they are anchored on (the founder's decision of
//! 2026-10-22, v4changelog known issue 20): no plan item can carry a span no
//! word has (D-121). The other four files are Prompt 93's, unchanged.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use offcut_detect::{DetectorConfig, detect};
use offcut_plan::{PlanConfig, from_rules, validate};
use offcut_scene::{FontSet, Scene, SceneInput, Theme, build_scene};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{
    BitsPerSec, ClipInfo, Confidence, DetectedEvent, DurMs, EditState, EventId, EventKind,
    EventParams, ExportProfile, FrameIdx, Hz, Orientation, ProfileKind, Prosody, Px, Quantity,
    Rotation, Span, StyleId, TimeMap, TimeMs, Transcript, Unit, WordIdx, WordProsody, WordRange,
};
use serde_json::Value;

/// Every 7th frame: 0, 7, 14, … ("Gaps settled" item 3 of the V4 prompts).
const STEP: u32 = 7;

/// The names of V3's four fonts, by index: what `FontKey(n)` reads as (D-132).
const FONT_IDS: [&str; 4] = ["Inter700", "Inter900", "JetBrainsMono700", "NotoEmoji"];

struct Input {
    name: &'static str,
    transcript: Transcript,
    events: Vec<DetectedEvent>,
    clip: ClipInfo,
    width: u32,
}

// --- The helpers of `offcut-scene`'s `lib.rs` tests, copied (a test file
// cannot reach `#[cfg(test)]` code), on the V4 contract. --------------------

fn word(text: &str, start_ms: u32, end_ms: u32) -> RawWord {
    RawWord {
        text: text.to_owned(),
        start_ms,
        end_ms,
        confidence: 1.0,
    }
}

fn transcript(words: &[RawWord]) -> Transcript {
    normalize_transcript(words.to_vec(), "test")
}

fn clip(width: u32, height: u32, duration_ms: u32) -> ClipInfo {
    ClipInfo {
        duration: DurMs::new(duration_ms),
        display_width: Px::new(width),
        display_height: Px::new(height),
        rotation: Rotation::R0,
        orientation: if width > height {
            Orientation::Landscape
        } else {
            Orientation::Portrait
        },
        is_vfr: false,
        video_codec_string: "avc1.640028".to_owned(),
        audio_codec_string: "mp4a.40.2".to_owned(),
        audio_sample_rate: Hz::new(48_000),
        audio_channels: 2,
    }
}

fn profile(width: u32) -> ExportProfile {
    ExportProfile {
        kind: ProfileKind::Creator,
        width: Px::new(width),
        height: Px::new(width * 16 / 9),
        watermark: false,
        video_bitrate: BitsPerSec::new(8_000_000),
        audio_bitrate: BitsPerSec::new(160_000),
    }
}

/// A NumberReveal on the word `word`. `validate` takes its span from that
/// word's times (D-121); `start` and `end` are that word's span, the span
/// V3's scene drew the reference with.
fn number_event(value: Quantity, word: u32, start: u32, end: u32) -> DetectedEvent {
    DetectedEvent {
        id: EventId::new(u64::from(start)),
        kind: EventKind::NumberReveal,
        span: Span::new(TimeMs::new(start), TimeMs::new(end)).unwrap(),
        anchors: WordRange {
            start: WordIdx::new(word),
            end: WordIdx::new(word + 1),
        },
        params: EventParams::NumberReveal { value, label: None },
        confidence: Confidence::new(1.0).unwrap(),
        enabled: true,
    }
}

/// `busy()` of `lib.rs`'s tests: captions with a long and a wide word, a
/// pause, a number that counts up through texts wider than its end, and one
/// that scales in.
fn busy() -> (Transcript, Vec<DetectedEvent>) {
    let texts = [
        "we",
        "spent",
        "three",
        "long",
        "months",
        "building",
        "something",
        "WWWWWWWWWWWWWWWWWW",
        "internationalization",
        "nobody",
        "wanted.",
    ];
    let mut words: Vec<_> = (0u32..)
        .zip(texts)
        .map(|(i, text)| word(text, 500 + i * 300, 750 + i * 300))
        .collect();
    words.push(word("It", 6_000, 6_200));
    words.push(word("made", 6_250, 6_500));
    words.push(word("$250,000", 6_550, 7_000));
    words.push(word("here.", 7_050, 7_300));
    let quantity = |value: f64, unit: Unit, display: &str| Quantity {
        value,
        unit,
        display: display.to_owned(),
    };
    // The words `lib.rs`'s `busy()` puts them on (V4 Prompt 98): "three"
    // 1,100-1,350, "wanted." 3,500-3,750, "$250,000" 6,550-7,000.
    let events = vec![
        number_event(quantity(40.0, Unit::Percent, "40%"), 2, 1_100, 1_350),
        number_event(quantity(3.0, Unit::None, "3"), 10, 3_500, 3_750),
        number_event(quantity(250_000.0, Unit::Usd, "$250k"), 13, 6_550, 7_000),
    ];
    (transcript(&words), events)
}

// --- The reference clip (fixtures/speech/README.md, "The V2 reference clip
// as recognized"). ---------------------------------------------------------

/// The 157 recognized words. The README gives exact times for each
/// sentence's first start and last end and for the six number words (the
/// `$12` / `,000` split as in `offcut-detect`'s tests); it gives no other
/// word time. Every other word takes an equal share, rounded down, of the gap
/// between its fixed neighbours. The table is the input from now on: Prompt
/// 100 builds the same scenes from it.
#[rustfmt::skip]
const REFERENCE_WORDS: [(&str, u32, u32); 157] = [
    ("Last", 1_560, 1_987),
    ("year,", 1_987, 2_415),
    ("I", 2_415, 2_842),
    ("almost", 2_842, 3_270),
    ("quit", 3_270, 3_697),
    ("making", 3_697, 4_125),
    ("videos,", 4_125, 4_552),
    ("editing", 4_552, 4_980),
    ("8", 4_980, 6_180),
    ("every", 6_180, 6_630),
    ("evening", 6_630, 7_080),
    ("at", 7_080, 7_530),
    ("almost", 7_530, 7_980),
    ("nobody", 7_980, 8_430),
    ("watched.", 8_430, 8_880),
    ("So", 9_160, 9_404),
    ("I", 9_404, 9_648),
    ("added", 9_648, 9_892),
    ("it", 9_892, 10_136),
    ("up.", 10_136, 10_380),
    ("I", 10_580, 11_013),
    ("had", 11_013, 11_446),
    ("spent", 11_446, 11_880),
    ("3000", 11_880, 12_560),
    ("hours", 12_560, 12_945),
    ("editing", 12_945, 13_330),
    ("by", 13_330, 13_715),
    ("hand.", 13_715, 14_100),
    ("This", 14_260, 14_722),
    ("is", 14_722, 15_185),
    ("literally", 15_185, 15_647),
    ("brutal", 15_647, 16_110),
    ("way", 16_110, 16_572),
    ("to", 16_572, 17_035),
    ("make", 17_035, 17_497),
    ("anything.", 17_497, 17_960),
    ("The", 18_100, 18_555),
    ("slow", 18_555, 19_010),
    ("part", 19_010, 19_465),
    ("was", 19_465, 19_920),
    ("always", 19_920, 20_375),
    ("a", 20_375, 20_830),
    ("rough", 20_830, 21_285),
    ("cut.", 21_285, 21_740),
    ("Then", 22_100, 22_458),
    ("I", 22_458, 22_816),
    ("checked", 22_816, 23_174),
    ("my", 23_174, 23_532),
    ("workflow", 23_532, 23_890),
    ("and", 23_890, 24_249),
    ("the", 24_249, 24_607),
    ("rough", 24_607, 24_965),
    ("cut", 24_965, 25_323),
    ("went", 25_323, 25_681),
    ("from", 25_681, 26_040),
    ("90", 26_040, 26_320),
    ("minutes", 26_320, 26_570),
    ("to", 26_570, 26_820),
    ("10", 26_820, 27_180),
    ("minutes.", 27_180, 28_580),
    ("That", 29_040, 29_489),
    ("change", 29_489, 29_938),
    ("gave", 29_938, 30_387),
    ("me", 30_387, 30_836),
    ("my", 30_836, 31_286),
    ("evenings", 31_286, 31_735),
    ("back", 31_735, 32_184),
    ("and", 32_184, 32_633),
    ("honestly", 32_633, 33_083),
    ("it", 33_083, 33_532),
    ("was", 33_532, 33_981),
    ("not", 33_981, 34_430),
    ("luck.", 34_430, 34_880),
    ("There", 35_280, 35_680),
    ("are", 35_680, 36_080),
    ("three", 36_080, 36_400),
    ("reasons", 36_400, 36_866),
    ("it", 36_866, 37_333),
    ("worked.", 37_333, 37_800),
    ("First", 38_120, 38_632),
    ("the", 38_632, 39_144),
    ("captions", 39_144, 39_656),
    ("write", 39_656, 40_168),
    ("themselves.", 40_168, 40_680),
    ("Second", 41_080, 41_676),
    ("the", 41_676, 42_272),
    ("numbers", 42_272, 42_868),
    ("animate", 42_868, 43_464),
    ("automatically.", 43_464, 44_060),
    ("Third", 44_220, 44_616),
    ("everything", 44_616, 45_013),
    ("runs", 45_013, 45_410),
    ("on", 45_410, 45_806),
    ("my", 45_806, 46_203),
    ("laptop.", 46_203, 46_600),
    ("I", 46_840, 47_170),
    ("record", 47_170, 47_501),
    ("a", 47_501, 47_832),
    ("clip,", 47_832, 48_163),
    ("drop", 48_163, 48_494),
    ("it", 48_494, 48_825),
    ("in", 48_825, 49_156),
    ("and", 49_156, 49_487),
    ("read", 49_487, 49_818),
    ("the", 49_818, 50_149),
    ("result.", 50_149, 50_480),
    ("If", 50_580, 50_950),
    ("a", 50_950, 51_320),
    ("caption", 51_320, 51_690),
    ("is", 51_690, 52_060),
    ("wrong,", 52_060, 52_430),
    ("I", 52_430, 52_800),
    ("fix", 52_800, 53_170),
    ("the", 53_170, 53_540),
    ("word", 53_540, 53_910),
    ("and", 53_910, 54_280),
    ("move", 54_280, 54_650),
    ("on.", 54_650, 55_020),
    ("Now", 55_420, 56_035),
    ("I", 56_035, 56_651),
    ("publish", 56_651, 57_266),
    ("every", 57_266, 57_882),
    ("week", 57_882, 58_497),
    ("instead", 58_497, 59_113),
    ("of", 59_113, 59_728),
    ("every", 59_728, 60_344),
    ("month.", 60_344, 60_960),
    ("Last", 61_200, 61_492),
    ("month", 61_492, 61_784),
    ("those", 61_784, 62_076),
    ("clips", 62_076, 62_368),
    ("earned", 62_368, 62_660),
    ("$12", 62_660, 63_100),
    (",000", 63_100, 63_540),
    ("in", 63_540, 63_910),
    ("sales.", 63_910, 64_280),
    ("So", 64_360, 64_684),
    ("I", 64_684, 65_008),
    ("keep", 65_008, 65_332),
    ("talking", 65_332, 65_657),
    ("to", 65_657, 65_981),
    ("my", 65_981, 66_305),
    ("camera", 66_305, 66_630),
    ("and", 66_630, 66_954),
    ("let", 66_954, 67_278),
    ("the", 67_278, 67_602),
    ("software", 67_602, 67_927),
    ("handle", 67_927, 68_251),
    ("the", 68_251, 68_575),
    ("rest.", 68_575, 68_900),
    ("Your", 69_120, 69_462),
    ("ideas", 69_462, 69_805),
    ("matter", 69_805, 70_148),
    ("more", 70_148, 70_491),
    ("than", 70_491, 70_834),
    ("your", 70_834, 71_177),
    ("timeline.", 71_177, 71_520),
];

/// The reference clip: 1920 x 1080, 74,705 ms (fixtures/speech/README.md).
const REFERENCE_MS: u32 = 74_705;

fn reference() -> (Transcript, Vec<DetectedEvent>) {
    let words: Vec<_> = REFERENCE_WORDS
        .iter()
        .map(|&(text, start, end)| word(text, start, end))
        .collect();
    let t = transcript(&words);
    // No measured prosody: every word at zero (the README's "if the measured
    // prosody carries it" KeywordPop is therefore not found).
    let p = Prosody {
        per_word: vec![
            WordProsody {
                energy_z: 0.0,
                pitch_z: 0.0,
            };
            t.words.len()
        ],
    };
    let events = detect(&t, &p, &BTreeMap::new(), &DetectorConfig::default());
    (t, events)
}

// --- Labeled files, timed as `offcut-detect/tests/precision_recall.rs` does. --

/// `synthesize` of `precision_recall.rs` (v3implementation §17.5): a word
/// lasts `clamp(120 + 45 x letters, 150, 700)` ms; 80 ms between the words of
/// a sentence, 400 ms after a word ending in `.` `?` `!`; the first word
/// starts at 500 ms; `*word*` has energy_z and pitch_z 2.0, every other word
/// 0.0. Also returns where the next word would start, the clip's length.
fn synthesize(sentences: &[String]) -> (Vec<RawWord>, Prosody, u32) {
    let mut raw = Vec::new();
    let mut per_word = Vec::new();
    let mut at = 500u32;
    for text in sentences {
        for piece in text.split_whitespace() {
            let stressed = piece.starts_with('*')
                && piece.trim_end_matches(['.', ',', '?', '!']).ends_with('*');
            let word = piece.replace('*', "");
            let letters = word.chars().filter(|c| c.is_alphanumeric()).count() as u32;
            let length = (120 + 45 * letters).clamp(150, 700);
            raw.push(RawWord {
                text: word.clone(),
                start_ms: at,
                end_ms: at + length,
                confidence: 1.0,
            });
            let z = if stressed { 2.0 } else { 0.0 };
            per_word.push(WordProsody {
                energy_z: z,
                pitch_z: z,
            });
            at += length
                + if word.ends_with(['.', '?', '!']) {
                    400
                } else {
                    80
                };
        }
    }
    (raw, Prosody { per_word }, at)
}

/// A written labeled file: its transcript as `precision_recall.rs` builds it,
/// the detector's events with the file's emphasis marks, and its length.
fn labeled(file: &str) -> (Transcript, Vec<DetectedEvent>, u32) {
    let path = manifest_dir().join("../../fixtures/labeled").join(file);
    let text = std::fs::read_to_string(&path).expect("a labeled file of fixtures/labeled/");
    let json: Value = serde_json::from_str(&text).expect("a labeled file parses");
    assert_eq!(json["source"], "written", "{file}: a written file");
    let sentences: Vec<String> = json["sentences"]
        .as_array()
        .expect("sentences")
        .iter()
        .map(|s| s.as_str().expect("a sentence").to_owned())
        .collect();
    let (raw, p, length) = synthesize(&sentences);
    let t = normalize_transcript(raw, "labeled");
    let events = detect(&t, &p, &BTreeMap::new(), &DetectorConfig::default());
    (t, events, length)
}

// --- The six inputs (G 2.1). ---------------------------------------------------

fn inputs() -> Vec<Input> {
    let mut inputs = Vec::new();
    for (name, width) in [("busy_1080", 1080), ("busy_540", 540)] {
        let (transcript, events) = busy();
        inputs.push(Input {
            name,
            transcript,
            events,
            clip: clip(1080, 1920, 10_000),
            width,
        });
    }
    let (transcript, events) = reference();
    inputs.push(Input {
        name: "reference_1080",
        transcript,
        events,
        clip: clip(1920, 1080, REFERENCE_MS),
        width: 1080,
    });
    for (name, file) in [
        ("a01_1080", "a01-scriptA.json"),
        ("w01_1080", "w01-saas-pricing.json"),
        ("w07_1080", "w07-camera.json"),
    ] {
        let (transcript, events, length) = labeled(file);
        inputs.push(Input {
            name,
            transcript,
            events,
            clip: clip(1080, 1920, length),
            width: 1080,
        });
    }
    inputs
}

/// A list as V3 wrote it: without the leading `Video` command, and with
/// each glyph run's `FontKey(n)` as the name of V3's font of index `n` (D-117).
fn normalize(mut list: Value) -> Value {
    let Some(cmds) = list.get_mut("cmds").and_then(Value::as_array_mut) else {
        return list;
    };
    if cmds.first().is_some_and(|cmd| cmd.get("Video").is_some()) {
        cmds.remove(0);
    }
    for cmd in cmds.iter_mut() {
        let Some(font) = cmd.get_mut("GlyphRun").and_then(|run| run.get_mut("font")) else {
            continue;
        };
        if let Some(name) = font.as_u64().and_then(|n| FONT_IDS.get(n as usize)) {
            *font = Value::from(*name);
        }
    }
    list
}

/// Every 7th frame's display list, as `serde_json` writes it and reads it
/// back, normalized.
fn frames(scene: &Scene) -> Vec<Value> {
    (0..scene.frame_count())
        .step_by(STEP as usize)
        .map(|n| {
            let list = scene.frame_at(FrameIdx::new(n).to_out_ms());
            let text = serde_json::to_string(&list).expect("a display list serializes");
            normalize(serde_json::from_str(&text).expect("and reads back"))
        })
        .collect()
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn reference_path(name: &str) -> PathBuf {
    manifest_dir()
        .join("tests/snapshots/v3_reference")
        .join(format!("{name}.json"))
}

/// The path of the first value that differs, if any.
fn first_difference(a: &Value, b: &Value, path: &str) -> Option<String> {
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => {
            let keys: std::collections::BTreeSet<&String> = a.keys().chain(b.keys()).collect();
            keys.into_iter()
                .find_map(|key| match (a.get(key), b.get(key)) {
                    (Some(x), Some(y)) => first_difference(x, y, &format!("{path}.{key}")),
                    (Some(_), None) | (None, Some(_)) | (None, None) => {
                        Some(format!("{path}.{key}"))
                    }
                })
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Some(format!("{path} (length {} vs {})", a.len(), b.len()));
            }
            a.iter()
                .zip(b)
                .enumerate()
                .find_map(|(i, (x, y))| first_difference(x, y, &format!("{path}[{i}]")))
        }
        (a, b) => (a != b).then(|| format!("{path} ({a} vs {b})")),
    }
}

#[test]
fn v3_reference() {
    let theme = Theme::default_for(StyleId::Clean);
    let fonts = FontSet::embedded();
    let edit = EditState::default();
    let (mut compared, mut differing) = (0usize, Vec::new());
    let inputs = inputs();
    for input in &inputs {
        let rules = from_rules(&input.events, &input.transcript);
        let (plan, dropped) = validate(
            rules,
            &input.transcript,
            &edit.word_edits,
            &PlanConfig::default(),
        )
        .expect("a rules plan is never refused");
        let scene = build_scene(SceneInput {
            transcript: &input.transcript,
            plan: &plan,
            edit: &edit,
            clip: &input.clip,
            profile: &profile(input.width),
            time_map: &TimeMap::identity(input.clip.duration),
            theme: &theme,
            fonts: &fonts,
        })
        .expect("the scene builds");
        let got = frames(&scene);
        let path = reference_path(input.name);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{}: no {}", input.name, path.display()));
        let want: Vec<Value> = serde_json::from_str(&text).expect("a reference file parses");
        let differ: Vec<(usize, String)> = if got.len() == want.len() {
            got.iter()
                .zip(&want)
                .enumerate()
                .filter_map(|(i, (g, w))| first_difference(w, g, "").map(|at| (i, at)))
                .collect()
        } else {
            vec![(
                0,
                format!("{} frames now, {} in the reference", got.len(), want.len()),
            )]
        };
        compared += got.len();
        println!(
            "{}: {} frames of {}, {} items, {} dropped, {} differ",
            input.name,
            got.len(),
            scene.frame_count(),
            plan.items().len(),
            dropped.len(),
            differ.len()
        );
        for (i, at) in &differ {
            println!(
                "  frame {} (index {i}) at {at} (reference vs now)",
                *i as u32 * STEP
            );
        }
        if !differ.is_empty() {
            differing.push(input.name);
        }
    }
    assert!(
        differing.is_empty(),
        "{} inputs, {compared} frames; differ: {differing:?}",
        inputs.len()
    );
    println!("{} inputs, {compared} frames, identical", inputs.len());
}
