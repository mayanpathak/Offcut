//! The rules of v3implementation §9 and the edge cases of TS §17.6, one row
//! each (§19.11): written transcripts, timed as `precision_recall.rs` times
//! them (§17.5), through the whole `detect`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;

use offcut_detect::{DetectorConfig, detect};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{
    DetectedEvent, EventKind, EventParams, Prosody, Transcript, WordIdx, WordProsody,
};

// --- Timing synthesis (§17.5), the same rule as precision_recall.rs. -------

/// A sentence, and the earliest time its first word may start: a pause in
/// front of it when that is later than the synthesis would put it.
type Line<'a> = (&'a str, u32);

/// A word lasts `clamp(120 + 45 x letters, 150, 700)` ms; 80 ms between the
/// words of a sentence, 400 ms after a word ending in `.` `?` `!`; the first
/// word starts at 500 ms. `*word*` is emphasized: energy_z and pitch_z 2.0
/// (§23 item 27), every other word 0.0. Confidence 1.0 (TE-2).
fn synthesize(lines: &[Line<'_>]) -> (Transcript, Prosody) {
    let mut raw = Vec::new();
    let mut per_word = Vec::new();
    let mut at = 500u32;
    for (text, earliest) in lines {
        at = at.max(*earliest);
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
    (normalize_transcript(raw, "labeled"), Prosody { per_word })
}

/// Sentences that follow one another without a pause of their own.
fn said(text: &str) -> Vec<Line<'_>> {
    vec![(text, 0)]
}

// --- What a row expects. ----------------------------------------------------

/// An event as a row names it: its kind and what it shows.
fn shown(t: &Transcript, e: &DetectedEvent) -> (EventKind, String) {
    let text = match &e.params {
        EventParams::NumberReveal { value, .. } => value.display.clone(),
        EventParams::FromTo { from, to, .. } => format!("{} -> {}", from.display, to.display),
        EventParams::ListReveal { count, header, .. } => format!("{count} [{header}]"),
        EventParams::KeywordPop { word } => {
            let text = &t.words[word.get() as usize].text;
            text.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase()
        }
    };
    (e.kind, text)
}

/// Runs `detect` and checks INV-6 for every event (row 31).
fn events(lines: &[Line<'_>], edits: &[(u32, &str)]) -> (Transcript, Vec<DetectedEvent>) {
    let (t, p) = synthesize(lines);
    let edits: BTreeMap<WordIdx, String> = edits
        .iter()
        .map(|(word, text)| (WordIdx::new(*word), (*text).to_owned()))
        .collect();
    let cfg = DetectorConfig::default();
    let found = detect(&t, &p, &edits, &cfg);
    for e in &found {
        assert!(e.confidence.get() >= cfg.threshold(e.kind), "INV-6: {e:?}");
    }
    (t, found)
}

use EventKind::{FromTo, KeywordPop, ListReveal, NumberReveal};

/// One row: its number in §19.11, its input, what it expects (kind, what is
/// shown, and the confidence where the row gives one).
struct Row<'a> {
    row: u32,
    lines: Vec<Line<'a>>,
    edits: Vec<(u32, &'a str)>,
    expect: Vec<(EventKind, &'a str, Option<f32>)>,
}

fn row<'a>(
    row: u32,
    lines: Vec<Line<'a>>,
    expect: Vec<(EventKind, &'a str, Option<f32>)>,
) -> Row<'a> {
    Row {
        row,
        lines,
        edits: Vec::new(),
        expect,
    }
}

const SCRIPT_A_LIST: &str = "There are three reasons it worked. First, the captions write themselves. \
     Second, the numbers animate automatically. Third, everything runs on my laptop.";

fn rows() -> Vec<Row<'static>> {
    vec![
        row(
            1,
            said("Last month those clips earned $12,000 in sales."),
            vec![(NumberReveal, "$12k", Some(1.0))],
        ),
        row(
            2,
            said("I had spent 3000 hours editing by hand."),
            vec![(NumberReveal, "3,000 hours", Some(1.0))],
        ),
        row(
            3,
            said("We have 3000 users."),
            vec![(NumberReveal, "3,000 users", Some(1.0))],
        ),
        row(4, said("I have 3 tools."), vec![]),
        row(
            5,
            said("It costs 40% less."),
            vec![(NumberReveal, "40%", Some(0.80))],
        ),
        row(
            6,
            said("About 900 people came."),
            vec![(NumberReveal, "900 people", Some(0.80))],
        ),
        row(7, said("About 900 came."), vec![]),
        row(
            8,
            said("The rough cut went from 90 minutes to 10 minutes."),
            vec![(FromTo, "90 minutes -> 10 minutes", Some(1.0))],
        ),
        row(
            9,
            said("Revenue went from $2k to $20k."),
            vec![(FromTo, "$2k -> $20k", Some(1.0))],
        ),
        row(
            10,
            said("$2k to $20k."),
            vec![(NumberReveal, "$2k", Some(1.0))],
        ),
        row(11, said("From 2019 to 2024 I worked there."), vec![]),
        row(
            12,
            said("Costs dropped 40% to 25%."),
            vec![(FromTo, "40% -> 25%", Some(0.90))],
        ),
        row(
            13,
            said(SCRIPT_A_LIST),
            vec![(ListReveal, "3 [3 REASONS]", Some(1.0))],
        ),
        row(
            14,
            said(
                "There are three reasons it worked. First, the captions write themselves. \
                      Second, the numbers animate automatically.",
            ),
            vec![],
        ),
        row(
            15,
            said("Here are 4 tips. 1, sleep early. 2, eat well. 3, ship daily. 4, rest often."),
            vec![(ListReveal, "4 [4 TIPS]", Some(0.90))],
        ),
        row(
            16,
            vec![
                ("First, open the app.", 0),
                ("Second, drop a clip.", 5_500),
                ("Third, read the result.", 10_500),
            ],
            vec![(ListReveal, "3 []", Some(0.85))],
        ),
        row(
            17,
            vec![
                ("First, open the app.", 0),
                ("Second, drop a clip.", 5_500),
                ("Third, read the result.", 25_500),
            ],
            vec![],
        ),
        row(
            18,
            said("It was the first time. Then the second one came."),
            vec![],
        ),
        row(
            19,
            said("That is literally *brutal*."),
            vec![(KeywordPop, "brutal", Some(1.0))],
        ),
        row(20, said("That is literally brutal."), vec![]),
        // Item 27 is decided (pitch_z 2.0): the score is 0.90, not on the threshold.
        row(
            21,
            said("*Workflow* matters."),
            vec![(KeywordPop, "workflow", Some(0.90))],
        ),
        row(22, said("Upgrade to Python 3.12 now."), vec![]),
        row(23, said("The iPhone 15 Pro is here."), vec![]),
        row(24, said("Meet me at 5 pm."), vec![]),
        row(25, said("Call 5551234 today."), vec![]),
        row(26, said("I ate 3 apples."), vec![]),
        row(
            27,
            said("This *workflow* is *insane*."),
            vec![(KeywordPop, "insane", Some(1.0))],
        ),
        // "workflow" starts at 500 ms, "insane" at 2,565 ms: 2,065 ms apart.
        row(
            28,
            said("*Workflow* matters. That is *insane*."),
            vec![(KeywordPop, "insane", Some(1.0))],
        ),
        row(
            29,
            vec![
                ("$1,000.", 0),
                ("$2,000.", 3_000),
                ("$3,000.", 5_500),
                ("$4,000.", 8_000),
            ],
            vec![
                (NumberReveal, "$1k", Some(1.0)),
                (NumberReveal, "$2k", Some(1.0)),
                (NumberReveal, "$3k", Some(1.0)),
            ],
        ),
        row(
            30,
            said(
                "Here are three lessons. First, we saved $10,000. Second, we hired help. \
                      Third, we shipped faster.",
            ),
            vec![(ListReveal, "3 [3 LESSONS]", Some(1.0))],
        ),
        // Row 31 is INV-6, checked in `events` for every row. Row 32 is its own test.
        Row {
            row: 33,
            lines: said(SCRIPT_A_LIST),
            edits: vec![(11, "")],
            expect: vec![],
        },
        row(34, vec![], vec![]),
    ]
}

#[test]
fn the_rows_of_section_19_11() {
    let mut failed = Vec::new();
    let all = rows();
    for r in &all {
        let (t, found) = events(&r.lines, &r.edits);
        let got: Vec<(EventKind, String, f32)> = found
            .iter()
            .map(|e| {
                let (kind, text) = shown(&t, e);
                (kind, text, e.confidence.get())
            })
            .collect();
        let matches = got.len() == r.expect.len()
            && got.iter().zip(&r.expect).all(
                |((kind, text, score), (want_kind, want_text, want_score))| {
                    kind == want_kind
                        && text == want_text
                        && want_score.is_none_or(|want| (score - want).abs() < 1e-6)
                },
            );
        if !matches {
            failed.push(format!(
                "row {}: found {got:?}, the table says {:?}",
                r.row, r.expect
            ));
        }
    }
    println!(
        "rules_table: {} rows of section 19.11 run here, {} failed",
        all.len(),
        failed.len()
    );
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

#[test]
fn row_1_and_2_labels() {
    let label = |text: &str| {
        let (_, found) = events(&said(text), &[]);
        match &found[0].params {
            EventParams::NumberReveal { label, .. } => label.clone(),
            other @ (EventParams::ListReveal { .. }
            | EventParams::FromTo { .. }
            | EventParams::KeywordPop { .. }) => panic!("{other:?}"),
        }
    };
    assert_eq!(
        label("Last month those clips earned $12,000 in sales.").as_deref(),
        Some("in sales")
    );
    assert_eq!(
        label("I had spent 3000 hours editing by hand.").as_deref(),
        Some("editing by hand")
    );
}

#[test]
fn row_13_shows_three_items() {
    let (_, found) = events(&said(SCRIPT_A_LIST), &[]);
    match &found[0].params {
        EventParams::ListReveal { items, .. } => {
            let texts: Vec<&str> = items.iter().map(|i| i.text.as_str()).collect();
            assert_eq!(
                texts,
                [
                    "the captions write themselves",
                    "the numbers animate automatically",
                    "everything runs on my laptop"
                ]
            );
        }
        other @ (EventParams::NumberReveal { .. }
        | EventParams::FromTo { .. }
        | EventParams::KeywordPop { .. }) => panic!("{other:?}"),
    }
}

#[test]
fn row_32_an_edited_number_is_read_again_and_keeps_its_id() {
    let text = "Last month those clips earned $12,000 in sales.";
    let (_, before) = events(&said(text), &[]);
    let (t, after) = events(&said(text), &[(5, "$20,000")]);
    assert_eq!(after.len(), 1);
    assert_eq!(shown(&t, &after[0]), (NumberReveal, "$20k".to_owned()));
    // Open item 14, decided 2026-10-18: the same kind and anchors, the same id,
    // so the user's toggle carries over.
    assert_eq!(after[0].id, before[0].id);
}

#[test]
fn row_28_the_two_pops_are_under_the_gap_apart() {
    let (t, _) = synthesize(&said("*Workflow* matters. That is *insane*."));
    let gap = t.words[4].start_ms.get() - t.words[0].start_ms.get();
    assert!(gap < DetectorConfig::default().keyword_min_gap_ms, "{gap}");
}
