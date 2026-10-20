#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The authority for parsing and display (v3implementation §8.5, §19.8): each
//! row is the words as recognized, the tokens consumed, the value, the unit and
//! the display. A failing row is a bug of the grammar, never of the row.

use std::collections::BTreeMap;

use offcut_text::{format_quantity, parse_quantity, tokenize};
use offcut_types::{Confidence, TimeMs, Unit, Word};

/// Words one after the other, as a recognizer gives them.
fn words(text: &str) -> Vec<Word> {
    (0u32..)
        .zip(text.split_whitespace())
        .map(|(i, w)| Word {
            text: w.to_owned(),
            start_ms: TimeMs::new(i * 400),
            end_ms: TimeMs::new(i * 400 + 300),
            confidence: Confidence::new(1.0).unwrap(),
        })
        .collect()
}

/// The unit a row names: `none`, `usd`, …, `count <noun>`.
fn unit(name: &str) -> Unit {
    match name {
        "none" => Unit::None,
        "usd" => Unit::Usd,
        "eur" => Unit::Eur,
        "gbp" => Unit::Gbp,
        "inr" => Unit::Inr,
        "percent" => Unit::Percent,
        "times" => Unit::Times,
        "milliseconds" => Unit::Milliseconds,
        "seconds" => Unit::Seconds,
        "minutes" => Unit::Minutes,
        "hours" => Unit::Hours,
        "days" => Unit::Days,
        "bytes" => Unit::Bytes,
        "megabytes" => Unit::Megabytes,
        "gigabytes" => Unit::Gigabytes,
        other => match other.strip_prefix("count ") {
            Some(noun) => Unit::Count {
                noun: noun.to_owned(),
            },
            None => panic!("a row names an unknown unit: {other}"),
        },
    }
}

type ParseRow = (
    &'static str,
    Option<(usize, f64, &'static str, &'static str)>,
);

#[rustfmt::skip]
const PARSE_ROWS: [ParseRow; 62] = [
    ("10,000", Some((1, 10_000.0, "none", "10,000"))),
    ("3.5", Some((1, 3.5, "none", "3.5"))),
    ("$10,000", Some((1, 10_000.0, "usd", "$10k"))),
    ("$1,500", Some((1, 1_500.0, "usd", "$1,500"))),
    ("$950", Some((1, 950.0, "usd", "$950"))),
    ("$9.99", Some((1, 9.99, "usd", "$9.99"))),
    ("40%", Some((1, 40.0, "percent", "40%"))),
    ("12.5%", Some((1, 12.5, "percent", "12.5%"))),
    ("10k", Some((1, 10_000.0, "none", "10,000"))),
    ("$10k", Some((1, 10_000.0, "usd", "$10k"))),
    ("$1.5m", Some((1, 1_500_000.0, "usd", "$1.5m"))),
    ("$2m", Some((1, 2_000_000.0, "usd", "$2m"))),
    ("$1.25m", Some((1, 1_250_000.0, "usd", "$1.25m"))),
    ("$3b", Some((1, 3_000_000_000.0, "usd", "$3b"))),
    ("€500", Some((1, 500.0, "eur", "€500"))),
    ("£250k", Some((1, 250_000.0, "gbp", "£250k"))),
    ("₹12,345", Some((1, 12_345.0, "inr", "₹12,345"))),
    ("3x", Some((1, 3.0, "times", "3x"))),
    ("ten times faster", Some((2, 10.0, "times", "10x"))),
    ("three times", Some((1, 3.0, "none", "3"))),
    ("ten thousand", Some((2, 10_000.0, "none", "10,000"))),
    ("two hundred and fifty", Some((4, 250.0, "none", "250"))),
    ("ten million", Some((2, 10_000_000.0, "none", "10 million"))),
    ("2 million", Some((2, 2_000_000.0, "none", "2 million"))),
    ("1.5 billion", Some((2, 1_500_000_000.0, "none", "1.5 billion"))),
    ("three point five", Some((3, 3.5, "none", "3.5"))),
    ("three point five million", Some((4, 3_500_000.0, "none", "3.5 million"))),
    ("zero point zero five", Some((4, 0.05, "none", "0.05"))),
    ("a thousand", Some((2, 1_000.0, "none", "1,000"))),
    ("a hundred people", Some((3, 100.0, "count people", "100 people"))),
    ("five grand", Some((2, 5_000.0, "usd", "$5k"))),
    ("twelve thousand dollars", Some((3, 12_000.0, "usd", "$12k"))),
    ("12,000 dollars", Some((2, 12_000.0, "usd", "$12k"))),
    ("$12 ,000", Some((3, 12_000.0, "usd", "$12k"))),
    ("$500 dollars", Some((2, 500.0, "usd", "$500"))),
    ("$500 euros", Some((1, 500.0, "usd", "$500"))),
    ("40 percent", Some((2, 40.0, "percent", "40%"))),
    ("40 per cent", Some((3, 40.0, "percent", "40%"))),
    ("3000 hours", Some((2, 3_000.0, "hours", "3,000 hours"))),
    ("1 hour", Some((2, 1.0, "hours", "1 hour"))),
    ("90 minutes", Some((2, 90.0, "minutes", "90 minutes"))),
    ("800 ms", Some((2, 800.0, "milliseconds", "800 ms"))),
    ("thirty seconds", Some((2, 30.0, "seconds", "30 seconds"))),
    ("1.5 days", Some((2, 1.5, "days", "1.5 days"))),
    ("500 MB", Some((2, 500.0, "megabytes", "500 MB"))),
    ("2 gigabytes", Some((2, 2.0, "gigabytes", "2 GB"))),
    ("512 bytes", Some((2, 512.0, "bytes", "512 bytes"))),
    ("10,000 subscribers", Some((2, 10_000.0, "count subscribers", "10,000 subscribers"))),
    ("2 million views", Some((3, 2_000_000.0, "count views", "2 million views"))),
    ("3 reasons", Some((1, 3.0, "none", "3"))),
    ("ten pounds", Some((1, 10.0, "none", "10"))),
    ("2 TB", Some((1, 2.0, "none", "2"))),
    ("forty-two", Some((1, 42.0, "none", "42"))),
    ("fifteen hundred", Some((2, 1_500.0, "none", "1,500"))),
    ("one", Some((1, 1.0, "none", "1"))),
    ("half", None),
    ("5-10", None),
    ("10:30", None),
    ("$3x", None),
    ("₹2,00,000", None),
    ("million", None),
    ("the", None),
];

#[test]
fn every_parse_row_of_the_table() {
    let mut failed = Vec::new();
    for (text, expected) in PARSE_ROWS {
        let tokens = tokenize(&words(text), &BTreeMap::new());
        let found = parse_quantity(&tokens).map(|(used, q)| (used, q.value, q.unit, q.display));
        let expected =
            expected.map(|(used, value, u, display)| (used, value, unit(u), display.to_owned()));
        if found != expected {
            failed.push(format!(
                "{text:?}: found {found:?}, the table says {expected:?}"
            ));
        }
    }
    assert!(
        failed.is_empty(),
        "{} rows fail:\n{}",
        failed.len(),
        failed.join("\n")
    );
}

#[rustfmt::skip]
const DISPLAY_ROWS: [(&str, f64, &str); 10] = [
    ("none", 999_999.0, "999,999"),
    ("none", 1_000_000.0, "1 million"),
    ("usd", 1_000_000.0, "$1m"),
    ("usd", 999_000.0, "$999k"),
    ("gbp", 1_000.0, "£1k"),
    ("percent", 0.1, "0.1%"),
    ("bytes", 1.0, "1 byte"),
    ("minutes", 1.0, "1 minute"),
    ("times", 2.25, "2.25x"),
    ("usd", 999e12, "$999t"),
];

#[test]
fn every_display_row_of_the_table() {
    let mut failed = Vec::new();
    for (u, value, expected) in DISPLAY_ROWS {
        let found = format_quantity(value, &unit(u));
        if found != expected {
            failed.push(format!(
                "({value}, {u}): found {found:?}, the table says {expected:?}"
            ));
        }
    }
    assert!(
        failed.is_empty(),
        "{} rows fail:\n{}",
        failed.len(),
        failed.join("\n")
    );
}
