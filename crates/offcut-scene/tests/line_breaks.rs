//! The line-break table (v4implementation D-167, §8.10; G 8.1, G item 6):
//! thirty sentences and how a person would break each into caption chunks
//! and lines, in Clean (16 characters x 2 lines, 4 words, as spoken; D-170)
//! and Bold (12 x 2, 3 words, upper case). Every expectation was decided by
//! reading the sentence aloud, before any break search existed (V4 Prompt
//! 102); from the table's commit on, a row changes only with a changelog line
//! naming the row and the reason (§23).
//!
//! Each sentence is spoken at 300 ms a word, the words one after the other,
//! with a 400 ms silence before each word `pause_before` names (a word index,
//! from 0). A line holds caption words: a number with its unit is one word,
//! shown as its display (TS §17.2), so `3 hours` and `$20k` are whole. A line
//! of one word may be longer than its style's characters per line: it is
//! scaled to fit (V2's rule).
//!
//! `table` compares the break search (V4 Prompt 103) with every row; the
//! property checks run it over the 31 labeled transcripts (§8.10): every word
//! once and in order, no chunk over its style's limits, and no line ended on
//! a binder that a layout could have avoided.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;
use std::ops::Range;
use std::path::PathBuf;

use offcut_scene::captions::{CHUNK_GAP_MS, CaptionWord, Chunk, MAX_SEGMENT_WORDS, chunk, words};
use offcut_scene::readability::binds_to_next;
use offcut_scene::styles::{Case, StyleSpec, spec};
use offcut_scene::{FontSet, Theme};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{DurMs, StyleId, TimeMap, Transcript, WordIdx};
use serde_json::Value;

/// How long each word is spoken, and the silence a row's pauses leave.
const WORD_MS: u32 = 300;
const PAUSE_MS: u32 = 400;

struct Row {
    id: u8,
    sentence: &'static str,
    pause_before: &'static [usize],
    /// Chunks, each its lines.
    clean: &'static [&'static [&'static str]],
    bold: &'static [&'static [&'static str]],
}

#[rustfmt::skip]
const ROWS: [Row; 30] = [
    // An article never ends a line or a chunk.
    Row { id: 1, sentence: "Today we shipped the new app.", pause_before: &[],
        clean: &[&["Today we shipped"], &["the new app."]],
        bold: &[&["TODAY WE", "SHIPPED"], &["THE NEW APP."]] },
    Row { id: 2, sentence: "I bought a camera for Japan.", pause_before: &[],
        clean: &[&["I bought", "a camera"], &["for Japan."]],
        bold: &[&["I BOUGHT"], &["A CAMERA"], &["FOR JAPAN."]] },
    Row { id: 3, sentence: "Grab the link in my bio.", pause_before: &[],
        clean: &[&["Grab the link"], &["in my bio."]],
        bold: &[&["GRAB", "THE LINK"], &["IN MY BIO."]] },
    // A preposition never ends a line or a chunk.
    Row { id: 4, sentence: "We moved everything to Postgres.", pause_before: &[],
        clean: &[&["We moved", "everything"], &["to Postgres."]],
        bold: &[&["WE MOVED", "EVERYTHING"], &["TO POSTGRES."]] },
    Row { id: 5, sentence: "She waited for hours at Heathrow.", pause_before: &[],
        clean: &[&["She waited", "for hours"], &["at Heathrow."]],
        bold: &[&["SHE WAITED"], &["FOR HOURS"], &["AT HEATHROW."]] },
    Row { id: 6, sentence: "I uploaded the video to YouTube.", pause_before: &[],
        clean: &[&["I uploaded", "the video"], &["to YouTube."]],
        bold: &[&["I UPLOADED"], &["THE VIDEO"], &["TO YOUTUBE."]] },
    // A break after a comma.
    Row { id: 7, sentence: "Honestly, it was worth every minute.", pause_before: &[],
        clean: &[&["Honestly,", "it was worth"], &["every minute."]],
        bold: &[&["HONESTLY,"], &["IT WAS WORTH"], &["EVERY", "MINUTE."]] },
    Row { id: 8, sentence: "If it rains, we stay inside.", pause_before: &[],
        clean: &[&["If it rains,"], &["we stay inside."]],
        bold: &[&["IF IT RAINS,"], &["WE STAY", "INSIDE."]] },
    // A clause starter begins a chunk.
    Row { id: 9, sentence: "It worked because we cached it.", pause_before: &[],
        clean: &[&["It worked"], &["because we", "cached it."]],
        bold: &[&["IT WORKED"], &["BECAUSE WE"], &["CACHED IT."]] },
    Row { id: 10, sentence: "Call me when it arrives.", pause_before: &[],
        clean: &[&["Call me"], &["when it arrives."]],
        bold: &[&["CALL ME"], &["WHEN IT", "ARRIVES."]] },
    // Two lines of a chunk are balanced, not a full line and a short one.
    Row { id: 11, sentence: "We found my favorite coffee shop.", pause_before: &[],
        clean: &[&["We found"], &["my favorite", "coffee shop."]],
        bold: &[&["WE FOUND"], &["MY FAVORITE"], &["COFFEE SHOP."]] },
    Row { id: 12, sentence: "The team loved it.", pause_before: &[],
        clean: &[&["The team", "loved it."]],
        bold: &[&["THE TEAM"], &["LOVED IT."]] },
    // A one-word beat between two pauses stays alone.
    Row { id: 13, sentence: "And then... nothing.", pause_before: &[2],
        clean: &[&["And then..."], &["nothing."]],
        bold: &[&["AND THEN..."], &["NOTHING."]] },
    Row { id: 14, sentence: "We opened it wow it was gone.", pause_before: &[3, 4],
        clean: &[&["We opened it"], &["wow"], &["it was gone."]],
        bold: &[&["WE OPENED IT"], &["WOW"], &["IT WAS GONE."]] },
    // Otherwise no word is left alone.
    Row { id: 15, sentence: "We launched on Monday.", pause_before: &[],
        clean: &[&["We launched", "on Monday."]],
        bold: &[&["WE LAUNCHED"], &["ON MONDAY."]] },
    Row { id: 16, sentence: "See you next week.", pause_before: &[],
        clean: &[&["See you", "next week."]],
        bold: &[&["SEE YOU"], &["NEXT WEEK."]] },
    // A number stays whole: one caption word, with its unit.
    Row { id: 17, sentence: "We went from $2k to $20k in sales.", pause_before: &[],
        clean: &[&["We went from $2k"], &["to $20k", "in sales."]],
        bold: &[&["WE WENT"], &["FROM $2K"], &["TO $20K"], &["IN SALES."]] },
    Row { id: 18, sentence: "It took 3 hours to edit.", pause_before: &[],
        clean: &[&["It took 3 hours"], &["to edit."]],
        bold: &[&["IT TOOK", "3 HOURS"], &["TO EDIT."]] },
    // A sentence end is a hard break, even where one chunk would fit.
    Row { id: 19, sentence: "It works. Ship it.", pause_before: &[],
        clean: &[&["It works."], &["Ship it."]],
        bold: &[&["IT WORKS."], &["SHIP IT."]] },
    Row { id: 20, sentence: "We tried it. It failed.", pause_before: &[],
        clean: &[&["We tried it."], &["It failed."]],
        bold: &[&["WE TRIED IT."], &["IT FAILED."]] },
    // A word longer than a line has a line of its own, scaled to fit.
    Row { id: 21, sentence: "Internationalization matters.", pause_before: &[],
        clean: &[&["Internationalization", "matters."]],
        bold: &[&["INTERNATIONALIZATION", "MATTERS."]] },
    Row { id: 22, sentence: "Congratulations everybody!", pause_before: &[],
        clean: &[&["Congratulations", "everybody!"]],
        bold: &[&["CONGRATULATIONS", "EVERYBODY!"]] },
    // Everyday creator speech (D-177): a recipe step, a fitness cue, a
    // product review line, a travel tip, a reaction.
    Row { id: 23, sentence: "Season it, then stir gently.", pause_before: &[],
        clean: &[&["Season it,"], &["then stir", "gently."]],
        bold: &[&["SEASON IT,"], &["THEN STIR", "GENTLY."]] },
    Row { id: 24, sentence: "Breathe in, now hold it.", pause_before: &[],
        clean: &[&["Breathe in,"], &["now hold it."]],
        bold: &[&["BREATHE IN,"], &["NOW HOLD IT."]] },
    Row { id: 25, sentence: "The battery lasts all day, which is great.", pause_before: &[],
        clean: &[&["The battery"], &["lasts all day,"], &["which is great."]],
        bold: &[&["THE BATTERY"], &["LASTS", "ALL DAY,"], &["WHICH", "IS GREAT."]] },
    Row { id: 26, sentence: "Pack light and arrive early.", pause_before: &[],
        clean: &[&["Pack light"], &["and arrive", "early."]],
        bold: &[&["PACK LIGHT"], &["AND ARRIVE", "EARLY."]] },
    Row { id: 27, sentence: "Oh wow, I love this!", pause_before: &[],
        clean: &[&["Oh wow,"], &["I love this!"]],
        bold: &[&["OH WOW,"], &["I LOVE THIS!"]] },
    // Longer sentences with several rules at once.
    Row { id: 28, sentence: "We spent months building it, and nobody used it.", pause_before: &[],
        clean: &[&["We spent months"], &["building it,"], &["and nobody", "used it."]],
        bold: &[&["WE SPENT", "MONTHS"], &["BUILDING IT,"], &["AND NOBODY"], &["USED IT."]] },
    Row { id: 29, sentence: "So I opened the app and it crashed.", pause_before: &[5],
        clean: &[&["So I opened"], &["the app"], &["and it crashed."]],
        bold: &[&["SO I OPENED"], &["THE APP"], &["AND IT", "CRASHED."]] },
    Row { id: 30, sentence: "This camera is small but powerful.", pause_before: &[],
        clean: &[&["This camera", "is small"], &["but powerful."]],
        bold: &[&["THIS CAMERA"], &["IS SMALL"], &["BUT", "POWERFUL."]] },
];

/// A style's limits, as the table reads them: the rest of the row is
/// `styles::spec`'s, which shapes and places but does not break.
struct Limits {
    style: StyleId,
    case: Case,
    chars: u32,
    lines: u32,
    words: u32,
}

const CLEAN: Limits = Limits {
    style: StyleId::Clean,
    case: Case::AsSpoken,
    chars: 16,
    lines: 2,
    words: 4,
};
const BOLD: Limits = Limits {
    style: StyleId::Bold,
    case: Case::Upper,
    chars: 12,
    lines: 2,
    words: 3,
};

impl Limits {
    fn spec(&self) -> StyleSpec {
        let mut row = spec(self.style);
        row.case = self.case;
        row.max_chars_per_line = self.chars;
        row.max_lines = self.lines;
        row.max_words = self.words;
        row
    }
}

/// The row's sentence as the pipeline transcribes it.
fn transcript(row: &Row) -> (Transcript, u32) {
    let mut at = 0u32;
    let mut raw = Vec::new();
    for (index, text) in row.sentence.split_whitespace().enumerate() {
        if row.pause_before.contains(&index) {
            at += PAUSE_MS;
        }
        raw.push(RawWord {
            text: text.to_owned(),
            start_ms: at,
            end_ms: at + WORD_MS,
            confidence: 1.0,
        });
        at += WORD_MS;
    }
    (normalize_transcript(raw, "line_breaks"), at)
}

/// The chunks and lines `captions::chunk` makes of the row, as text.
fn broken(row: &Row, limits: &Limits) -> Vec<Vec<String>> {
    let (t, length) = transcript(row);
    let map = TimeMap::identity(DurMs::new(length + 1_000));
    let shown = words(&t, &BTreeMap::new(), &map);
    let theme = Theme::default_for(limits.style);
    let chunks = chunk(
        &shown,
        &t.sentences,
        &limits.spec(),
        &theme,
        &FontSet::embedded(),
    )
    .unwrap();
    chunks
        .iter()
        .map(|c| {
            c.lines
                .iter()
                .map(|line| {
                    let text: Vec<String> = line
                        .words
                        .iter()
                        .map(|w| match limits.case {
                            Case::AsSpoken => c.words[w.word].text.clone(),
                            Case::Upper => c.words[w.word].text.to_uppercase(),
                        })
                        .collect();
                    text.join(" ")
                })
                .collect()
        })
        .collect()
}

/// A row's expectation keeps its own style's lines, characters and case.
/// (Its word limit counts caption words, `3 hours` being one: the
/// comparison holds it, since `chunk` never makes a longer chunk.)
fn assert_fits(number: u8, expected: &[&[&str]], limits: &Limits) {
    for lines in expected {
        assert!(
            lines.len() as u32 <= limits.lines,
            "row {number}: {lines:?}"
        );
        for line in *lines {
            let one_word = !line.contains(' ');
            assert!(
                one_word || line.chars().count() as u32 <= limits.chars,
                "row {number}: {line:?}"
            );
            if let Case::Upper = limits.case {
                assert_eq!(*line, line.to_uppercase(), "row {number}");
            }
        }
    }
}

#[test]
fn table() {
    let mut differ = Vec::new();
    for row in &ROWS {
        for (name, expected, limits) in [("Clean", row.clean, &CLEAN), ("Bold", row.bold, &BOLD)] {
            assert_fits(row.id, expected, limits);
            let want: Vec<Vec<String>> = expected
                .iter()
                .map(|lines| lines.iter().map(|l| (*l).to_owned()).collect())
                .collect();
            let got = broken(row, limits);
            if got != want {
                differ.push(format!("row {} {name}: want {want:?}, got {got:?}", row.id));
            }
        }
    }
    for line in &differ {
        println!("{line}");
    }
    assert!(differ.is_empty(), "{} of 60 differ", differ.len());
}

// --- The property checks over the labeled set (§8.10) ----------------------

/// A labeled transcript, timed as `offcut-detect`'s `precision_recall.rs`
/// times a written one, with the file's word edits.
struct Labeled {
    name: String,
    t: Transcript,
    edits: BTreeMap<WordIdx, String>,
}

/// `synthesize` of `precision_recall.rs` (v3implementation §17.5), without the
/// prosody: a word lasts `clamp(120 + 45 x letters, 150, 700)` ms, 80 ms
/// between the words of a sentence, 400 ms after a word ending in `.` `?` `!`.
fn synthesize(sentences: &[String]) -> Vec<RawWord> {
    let mut raw = Vec::new();
    let mut at = 500u32;
    for text in sentences {
        for piece in text.split_whitespace() {
            let word = piece.replace('*', "");
            let letters = word.chars().filter(|c| c.is_alphanumeric()).count() as u32;
            let length = (120 + 45 * letters).clamp(150, 700);
            raw.push(RawWord {
                text: word.clone(),
                start_ms: at,
                end_ms: at + length,
                confidence: 1.0,
            });
            at += length
                + if word.ends_with(['.', '?', '!']) {
                    400
                } else {
                    80
                };
        }
    }
    raw
}

/// The files of `fixtures/labeled/`, as Prompt 97's tests read them.
fn labeled() -> Vec<Labeled> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/labeled");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("fixtures/labeled")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            let json: Value =
                serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            let raw = match json["source"].as_str() {
                Some("written") => synthesize(
                    &serde_json::from_value::<Vec<String>>(json["sentences"].clone()).unwrap(),
                ),
                Some("asr") => json["words"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|w| RawWord {
                        text: w["text"].as_str().unwrap().to_owned(),
                        start_ms: u32::try_from(w["startMs"].as_u64().unwrap()).unwrap(),
                        end_ms: u32::try_from(w["endMs"].as_u64().unwrap()).unwrap(),
                        confidence: w["confidence"].as_f64().unwrap() as f32,
                    })
                    .collect(),
                other => panic!("{name}: source {other:?}"),
            };
            let edits = json["edits"]
                .as_object()
                .map(|map| {
                    map.iter()
                        .map(|(word, text)| {
                            (
                                WordIdx::new(word.parse().unwrap()),
                                text.as_str().unwrap().to_owned(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            Labeled {
                t: normalize_transcript(raw, "labeled"),
                name,
                edits,
            }
        })
        .collect()
}

/// A caption word in the style's case.
fn styled(word: &CaptionWord, limits: &Limits) -> String {
    match limits.case {
        Case::AsSpoken => word.text.clone(),
        Case::Upper => word.text.to_uppercase(),
    }
}

/// The characters of `texts` set on one line.
fn width(texts: &[String]) -> u32 {
    let chars: usize = texts.iter().map(|t| t.chars().count()).sum();
    (chars + texts.len().saturating_sub(1)) as u32
}

/// A line of these words fits: within the characters, or one word.
fn line_fits(texts: &[String], limits: &Limits) -> bool {
    texts.len() == 1 || width(texts) <= limits.chars
}

/// The runs between hard breaks, as `chunk` cuts them: a sentence end, a
/// pause of `CHUNK_GAP_MS`, `MAX_SEGMENT_WORDS`.
fn runs(shown: &[CaptionWord], t: &Transcript) -> Vec<Range<usize>> {
    let sentence_of = |w: &CaptionWord| t.sentences.partition_point(|s| s.words.end <= w.word);
    let mut runs = Vec::new();
    let mut from = 0;
    for to in 1..=shown.len() {
        let hard = to == shown.len()
            || sentence_of(&shown[to - 1]) != sentence_of(&shown[to])
            || shown[to]
                .start
                .get()
                .saturating_sub(shown[to - 1].end.get())
                >= CHUNK_GAP_MS
            || to - from >= MAX_SEGMENT_WORDS;
        if hard {
            runs.push(from..to);
            from = to;
        }
    }
    runs
}

/// The fewest lines ended on a binder that any layout of the run within the
/// style's limits can have. A run's last word ends at a hard break and is not
/// counted. Worked out here, by a search of its own over every layout.
fn fewest_binder_ends(texts: &[String], limits: &Limits) -> u32 {
    let n = texts.len();
    let words = limits.words as usize;
    let mut best = vec![u32::MAX; n + 1];
    best[0] = 0;
    for i in 1..=n {
        for j in i.saturating_sub(words)..i {
            if best[j] == u32::MAX {
                continue;
            }
            let inside = if line_fits(&texts[j..i], limits) {
                Some(0)
            } else {
                (j + 1..i)
                    .filter(|&k| line_fits(&texts[j..k], limits) && line_fits(&texts[k..i], limits))
                    .map(|k| u32::from(binds_to_next(&texts[k - 1])))
                    .min()
            };
            let Some(inside) = inside else { continue };
            let end = u32::from(i < n && binds_to_next(&texts[i - 1]));
            best[i] = best[i].min(best[j] + inside + end);
        }
    }
    best[n]
}

/// The lines of `chunks` ended on a binder, those at the end of a run left out.
fn binder_ends(
    chunks: &[Chunk],
    run_ends: &[usize],
    shown: &[CaptionWord],
    limits: &Limits,
) -> u32 {
    let index_of = |w: &CaptionWord| {
        shown
            .iter()
            .position(|s| s.word == w.word && s.start == w.start)
            .unwrap()
    };
    let mut count = 0;
    for c in chunks {
        for line in &c.lines {
            let last = &c.words[line.words.last().unwrap().word];
            let ends_a_run = run_ends.contains(&(index_of(last) + 1));
            if !ends_a_run && binds_to_next(&styled(last, limits)) {
                count += 1;
            }
        }
    }
    count
}

#[test]
fn every_labeled_transcript_breaks_within_its_limits_and_on_no_avoidable_binder() {
    let files = labeled();
    assert_eq!(files.len(), 31, "the labeled set");
    let mut totals = Vec::new();
    for limits in [&CLEAN, &BOLD] {
        let (mut chunks_seen, mut binders_seen) = (0, 0);
        for file in &files {
            let length = file.t.words.last().map_or(0, |w| w.end_ms.get());
            let map = TimeMap::identity(DurMs::new(length + 1_000));
            let shown = words(&file.t, &file.edits, &map);
            let theme = Theme::default_for(limits.style);
            let chunks = chunk(
                &shown,
                &file.t.sentences,
                &limits.spec(),
                &theme,
                &FontSet::embedded(),
            )
            .unwrap();
            let name = &file.name;

            // Every word exactly once, and in order.
            let laid: Vec<&CaptionWord> = chunks.iter().flat_map(|c| c.words.iter()).collect();
            assert_eq!(laid.len(), shown.len(), "{name}");
            for (a, b) in laid.iter().zip(&shown) {
                assert_eq!(
                    (a.word, &a.text, a.start),
                    (b.word, &b.text, b.start),
                    "{name}"
                );
            }

            // No chunk over its style's limits.
            for c in &chunks {
                assert!(
                    c.words.len() <= limits.words as usize,
                    "{name}: {:?}",
                    c.words
                );
                assert!(
                    c.lines.len() as u32 <= limits.lines,
                    "{name}: {:?}",
                    c.words
                );
                for line in &c.lines {
                    let texts: Vec<String> = line
                        .words
                        .iter()
                        .map(|w| styled(&c.words[w.word], limits))
                        .collect();
                    assert!(line_fits(&texts, limits), "{name}: {texts:?}");
                }
            }

            // No line ended on a binder that another layout of its run avoids.
            let runs = runs(&shown, &file.t);
            let run_ends: Vec<usize> = runs.iter().map(|r| r.end).collect();
            let fewest: u32 = runs
                .iter()
                .map(|r| {
                    let texts: Vec<String> =
                        shown[r.clone()].iter().map(|w| styled(w, limits)).collect();
                    fewest_binder_ends(&texts, limits)
                })
                .sum();
            let made = binder_ends(&chunks, &run_ends, &shown, limits);
            assert_eq!(
                made, fewest,
                "{name}: {made} lines end on a binder, {fewest} must"
            );
            chunks_seen += chunks.len();
            binders_seen += made;
        }
        totals.push(format!(
            "{:?}: {chunks_seen} chunks, {binders_seen} unavoidable binder ends",
            limits.style
        ));
    }
    println!("31 files; {}", totals.join("; "));
}
