//! Spelled cardinals and "point" decimals (v3implementation §8.2, D-92): the
//! part of the number grammar that reads words. Private to the crate:
//! `numbers.rs` stays the only entry to parsing and the only formatter.
//!
//! ```text
//! cardinal  := group (magnitude group?)* | "a" ("hundred" | magnitude) …        ; "a thousand" = 1,000
//! group     := 0..99 | unit "hundred" ("and"? 1..99)? | teen "hundred" ("and"? 1..99)?   ; "fifteen hundred" = 1,500
//! 0..99     := "zero" | unit | teen | tens | tens "-" unit (one token) | tens unit (two tokens)
//! magnitude := "thousand" | "million" | "billion" | "trillion"   ; strictly descending; each at most once
//! ```
//!
//! "and" is consumed only when a number from 1 to 99 follows it. A teen
//! hundred ("fifteen hundred") takes no magnitude after it. The value is at
//! most 999,999,999,999,999. A sequence that is not a cardinal gives `None`:
//! the grammar never guesses.

use crate::units_lex::{MAGNITUDE_WORDS, lookup};

/// "nine hundred and ninety nine trillion nine hundred and ninety nine
/// billion …": the longest cardinal is under this many tokens.
pub(crate) const MAX_CARDINAL_TOKENS: usize = 24;

const UNITS: [&str; 9] = [
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
];
const TEENS: [&str; 10] = [
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
const TENS: [&str; 8] = [
    "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];
const DIGITS: [&str; 10] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
];

/// A spelled cardinal at the start of `words` (lower-cased word texts): the
/// tokens it uses and its value.
pub(crate) fn cardinal(words: &[&str]) -> Option<(usize, u64)> {
    let words = words.get(..words.len().min(MAX_CARDINAL_TOKENS))?;
    if words.first() == Some(&"zero") {
        return Some((1, 0));
    }
    let (mut used, mut total) = (0usize, 0u64);
    // The magnitude of the group before, in its power of 1,000 (no magnitude yet: above any).
    let mut above = f64::INFINITY;
    let mut first = true;
    loop {
        let rest = words.get(used..)?;
        let Some(g) = group(rest, first) else {
            return (!first).then_some((used, total));
        };
        first = false;
        let magnitude = rest
            .get(g.used)
            .and_then(|w| lookup(MAGNITUDE_WORDS, w))
            .filter(|m| *m < above && !g.teen_hundred);
        match magnitude {
            Some(m) => {
                total = total.checked_add(g.value.checked_mul(m as u64)?)?;
                used = used.checked_add(g.used + 1)?;
                above = m;
            }
            None => {
                // After a magnitude, a group followed by one that is not smaller
                // is the start of the next number ("one million two million").
                let next_is_magnitude = rest
                    .get(g.used)
                    .and_then(|w| lookup(MAGNITUDE_WORDS, w))
                    .is_some();
                if used > 0 && next_is_magnitude && !g.teen_hundred {
                    return Some((used, total));
                }
                return Some((used.checked_add(g.used)?, total.checked_add(g.value)?));
            }
        }
    }
}

/// "point" and one or more digit words after a cardinal: the tokens used and
/// the fraction (0.0 to 1.0). "point zero five" is 0.05.
pub(crate) fn point_fraction(words: &[&str]) -> Option<(usize, f64)> {
    let words = words.get(..words.len().min(MAX_CARDINAL_TOKENS))?;
    if words.first() != Some(&"point") {
        return None;
    }
    let digits: String = words
        .iter()
        .skip(1)
        .map_while(|w| DIGITS.iter().position(|d| d == w))
        .filter_map(|d| char::from_digit(d as u32, 10))
        .collect();
    if digits.is_empty() {
        return None;
    }
    let value: f64 = format!("0.{digits}").parse().ok()?;
    Some((1 + digits.len(), value))
}

/// One group of a cardinal: below 100, or a number of hundreds with an
/// optional part below 100 after it.
struct Group {
    used: usize,
    value: u64,
    /// "fifteen hundred": a teen hundred, which takes no magnitude after it.
    teen_hundred: bool,
}

/// The group at the start of `words`. Only the first group of a cardinal may
/// be "a" before "hundred" or a magnitude ("a hundred", "a thousand").
fn group(words: &[&str], first: bool) -> Option<Group> {
    let head = *words.first()?;
    let second = words.get(1).copied();
    if first && head == "a" {
        return match second {
            Some("hundred") => Some(hundreds(words, 1, false)),
            Some(w) if lookup(MAGNITUDE_WORDS, w).is_some() => Some(Group {
                used: 1,
                value: 1,
                teen_hundred: false,
            }),
            _ => None,
        };
    }
    // A later group may begin with an "and" that a number from 1 to 99 follows.
    if !first && head == "and" {
        let (used, value) = below_100(words.get(1..)?)?;
        return Some(Group {
            used: used + 1,
            value: u64::from(value),
            teen_hundred: false,
        });
    }
    if let (Some(count), Some("hundred")) = (below_20(head), second) {
        return Some(hundreds(words, count, count >= 10));
    }
    let (used, value) = below_100(words)?;
    Some(Group {
        used,
        value: u64::from(value),
        teen_hundred: false,
    })
}

/// `count` hundreds ("hundred" at `words[1]`), then an optional "and" and a
/// number from 1 to 99.
fn hundreds(words: &[&str], count: u32, teen_hundred: bool) -> Group {
    let base = u64::from(count) * 100;
    let rest = words.get(2..).unwrap_or(&[]);
    let skipped = usize::from(rest.first() == Some(&"and"));
    match rest.get(skipped..).and_then(below_100) {
        Some((used, value)) => Group {
            used: 2 + skipped + used,
            value: base + u64::from(value),
            teen_hundred,
        },
        None => Group {
            used: 2,
            value: base,
            teen_hundred,
        },
    }
}

/// The place of `word` in `names`, counted from `first` in steps of `step`.
fn named(names: &[&str], word: &str, first: u32, step: u32) -> Option<u32> {
    names
        .iter()
        .position(|n| *n == word)
        .map(|i| first + i as u32 * step)
}

fn unit(word: &str) -> Option<u32> {
    named(&UNITS, word, 1, 1)
}

fn tens(word: &str) -> Option<u32> {
    named(&TENS, word, 20, 10)
}

/// One to nineteen: what can stand before "hundred".
fn below_20(word: &str) -> Option<u32> {
    unit(word).or_else(|| named(&TEENS, word, 10, 1))
}

/// 1 to 99: "five", "fifteen", "fifty", "fifty five", "fifty-five".
fn below_100(words: &[&str]) -> Option<(usize, u32)> {
    let first = *words.first()?;
    if let Some((left, right)) = first.split_once('-') {
        return Some((1, tens(left)? + unit(right)?));
    }
    if let Some(value) = tens(first) {
        return Some(match words.get(1).and_then(|w| unit(w)) {
            Some(ones) => (2, value + ones),
            None => (1, value),
        });
    }
    below_20(first).map(|value| (1, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> Option<(usize, u64)> {
        cardinal(&text.split_whitespace().collect::<Vec<_>>())
    }

    /// The 13 rows of v3implementation §8.2.
    #[test]
    fn the_rows_of_the_plan() {
        assert_eq!(read("ten thousand"), Some((2, 10_000)));
        assert_eq!(read("two hundred and fifty"), Some((4, 250)));
        assert_eq!(
            read("three million two hundred thousand"),
            Some((5, 3_200_000))
        );
        assert_eq!(read("one point five"), Some((1, 1)));
        assert_eq!(point_fraction(&["point", "five"]), Some((2, 0.5)));
        assert_eq!(read("a hundred"), Some((2, 100)));
        assert_eq!(read("a"), None);
        assert_eq!(read("a lot"), None);
        assert_eq!(read("fifteen hundred"), Some((2, 1_500)));
        assert_eq!(read("fifteen hundred thousand"), Some((2, 1_500)));
        assert_eq!(read("million"), None);
        assert_eq!(read("thousand million"), None);
        assert_eq!(read("two thousand thousand"), Some((2, 2_000)));
        // The plan writes 6 tokens for this row: the words are five; with "and" they are six.
        assert_eq!(
            read("nine hundred ninety nine trillion"),
            Some((5, 999_000_000_000_000))
        );
        assert_eq!(
            read("nine hundred and ninety nine trillion"),
            Some((6, 999_000_000_000_000))
        );
        assert_eq!(read("zero point zero five"), Some((1, 0)));
        assert_eq!(point_fraction(&["point", "zero", "five"]), Some((3, 0.05)));
    }

    #[test]
    fn magnitudes_descend_and_and_needs_a_number_after_it() {
        assert_eq!(read("a thousand"), Some((2, 1_000)));
        assert_eq!(read("one thousand and five"), Some((4, 1_005)));
        assert_eq!(read("one thousand and so"), Some((2, 1_000)));
        assert_eq!(read("two hundred and then"), Some((2, 200)));
        assert_eq!(read("one million two million"), Some((2, 1_000_000)));
        assert_eq!(
            read("one billion two million three"),
            Some((5, 1_002_000_003))
        );
        assert_eq!(read("forty-two"), Some((1, 42)));
        assert_eq!(read("forty two thousand"), Some((3, 42_000)));
        assert_eq!(read("and five"), None);
        assert_eq!(point_fraction(&["point"]), None);
        assert_eq!(point_fraction(&["point", "ten"]), None);
    }

    #[test]
    fn the_largest_cardinal_fits_in_the_token_cap() {
        let largest = "nine hundred ninety nine trillion nine hundred ninety nine billion nine hundred ninety nine million nine hundred ninety nine thousand nine hundred ninety nine";
        assert_eq!(largest.split_whitespace().count(), MAX_CARDINAL_TOKENS);
        assert_eq!(read(largest), Some((24, 999_999_999_999_999)));
        // With every "and" it is 29 words: the cap stops it after the thousands.
        let with_and = "nine hundred and ninety nine trillion nine hundred and ninety nine billion nine hundred and ninety nine million nine hundred and ninety nine thousand nine hundred and ninety nine";
        assert_eq!(read(with_and), Some((24, 999_999_999_999_000)));
    }
}
