#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Five properties of the number grammar (v3implementation §19.9): round trips
//! through the formatter, spelled numbers read whole, no panic on any token
//! sequence, and a display that is always the formatter's.

use std::collections::BTreeMap;

use offcut_text::{format_quantity, parse_quantity, tokenize};
use offcut_types::{Confidence, Quantity, TimeMs, Unit, Word};
use proptest::prelude::*;

fn words(texts: &[String]) -> Vec<Word> {
    (0u32..)
        .zip(texts)
        .map(|(i, w)| Word {
            text: w.clone(),
            start_ms: TimeMs::new(i * 400),
            end_ms: TimeMs::new(i * 400 + 300),
            confidence: Confidence::new(1.0).unwrap(),
        })
        .collect()
}

fn parse(text: &str) -> Option<(usize, Quantity)> {
    let texts: Vec<String> = text.split_whitespace().map(str::to_owned).collect();
    parse_quantity(&tokenize(&words(&texts), &BTreeMap::new()))
}

/// `n` with thousands separators: `1,234,567`.
fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

const UNITS: [&str; 20] = [
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
];
const TENS: [&str; 10] = [
    "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];

/// 1 to 999 in words, without "and": "nine hundred ninety nine".
fn spell_group(n: u64, out: &mut Vec<&'static str>) {
    if n >= 100 {
        out.push(UNITS[(n / 100) as usize]);
        out.push("hundred");
    }
    let rest = n % 100;
    if rest >= 20 {
        out.push(TENS[(rest / 10) as usize]);
        if !rest.is_multiple_of(10) {
            out.push(UNITS[(rest % 10) as usize]);
        }
    } else if rest > 0 {
        out.push(UNITS[rest as usize]);
    }
}

/// A test-only speller for 0 to 999,999,999.
fn spell(n: u64) -> String {
    if n == 0 {
        return "zero".to_owned();
    }
    let mut out = Vec::new();
    for (scale, word) in [
        (1_000_000, Some("million")),
        (1_000, Some("thousand")),
        (1, None),
    ] {
        let group = n / scale % 1_000;
        if group > 0 {
            spell_group(group, &mut out);
            if let Some(word) = word {
                out.push(word);
            }
        }
    }
    out.join(" ")
}

/// The words of a drawn sequence.
fn drawn(picks: &[usize]) -> Vec<String> {
    picks.iter().map(|&i| VOCABULARY[i].to_owned()).collect()
}

/// Number words, digits, symbols and fillers: what a sequence is drawn from.
const VOCABULARY: [&str; 40] = [
    "zero",
    "one",
    "five",
    "twelve",
    "twenty",
    "ninety",
    "hundred",
    "thousand",
    "million",
    "billion",
    "trillion",
    "a",
    "an",
    "and",
    "point",
    "grand",
    "times",
    "faster",
    "percent",
    "per",
    "cent",
    "dollars",
    "euros",
    "hours",
    "views",
    "the",
    "so",
    "$12",
    ",000",
    "10k",
    "$1.5m",
    "3x",
    "40%",
    "€500",
    "1,000",
    "3.5",
    ".",
    ",",
    "5-10",
    "₹2,00,000",
];

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// Digits with separators read back to the same value, with no unit.
    #[test]
    fn digits_round_trip(n in 0u64..=1_000_000_000_000) {
        let (used, q) = parse(&grouped(n)).unwrap();
        prop_assert_eq!(used, 1);
        prop_assert_eq!(q.value, n as f64);
        prop_assert_eq!(q.unit, Unit::None);
    }

    /// A dollar amount as the formatter writes it reads back to the same value
    /// when the display is exact, and within 0.5% when it rounds (`k`, `m`).
    #[test]
    fn currency_round_trip(n in 1u64..=1_000_000_000) {
        let display = format_quantity(n as f64, &Unit::Usd);
        let (_, q) = parse(&display).unwrap();
        prop_assert_eq!(&q.unit, &Unit::Usd);
        let exact = n < 1_000 || (n <= 999_000 && n.is_multiple_of(1_000)) || n.is_multiple_of(1_000_000);
        if exact {
            prop_assert_eq!(q.value, n as f64, "{}", display);
        } else {
            prop_assert!((q.value - n as f64).abs() <= n as f64 * 0.005, "{} read as {}", display, q.value);
        }
    }

    /// A spelled cardinal is read whole: every word is consumed.
    #[test]
    fn spelled_round_trip(n in 0u64..=999_999_999) {
        let text = spell(n);
        let (used, q) = parse(&text).unwrap();
        prop_assert_eq!(used, text.split_whitespace().count(), "{}", text);
        prop_assert_eq!(q.value, n as f64, "{}", text);
    }

    /// No sequence makes the parser panic; what it reads is in range and finite.
    #[test]
    fn no_panic(picks in prop::collection::vec(0..VOCABULARY.len(), 0..=30)) {
        let tokens = tokenize(&words(&drawn(&picks)), &BTreeMap::new());
        for start in 0..tokens.len() {
            let rest = &tokens[start..];
            if let Some((n, q)) = parse_quantity(rest) {
                prop_assert!(n >= 1 && n <= rest.len(), "{} tokens of {}", n, rest.len());
                prop_assert!(q.value.is_finite());
            }
        }
    }

    /// Whatever is read is displayed by the formatter, nothing else.
    #[test]
    fn the_display_is_the_formatters(picks in prop::collection::vec(0..VOCABULARY.len(), 1..=30)) {
        let tokens = tokenize(&words(&drawn(&picks)), &BTreeMap::new());
        for start in 0..tokens.len() {
            if let Some((_, q)) = parse_quantity(&tokens[start..]) {
                prop_assert_eq!(q.display, format_quantity(q.value, &q.unit));
            }
        }
    }
}

#[test]
fn the_speller_writes_what_it_should() {
    assert_eq!(spell(0), "zero");
    assert_eq!(spell(42), "forty two");
    assert_eq!(spell(1_500), "one thousand five hundred");
    assert_eq!(spell(999_999_999).split_whitespace().count(), 14);
}
