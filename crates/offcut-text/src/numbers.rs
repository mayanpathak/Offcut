//! Reading a number out of tokens, and writing one for a caption. This is
//! the only parser of spoken quantities and the only formatter (TS §17.1,
//! §17.2; v3implementation §8.3, §8.5).
//!
//! `parse_quantity` reads one quantity at the start of the tokens in four
//! stages: a core (digits with their sign or suffix, or a spelled cardinal
//! with an optional "point" fraction), a magnitude word, "grand", and one
//! unit. A sequence it cannot read gives `None`: it never guesses.

use offcut_types::{Quantity, Unit};

use crate::cardinals::{MAX_CARDINAL_TOKENS, cardinal, point_fraction};
use crate::tokenize::{Token, TokenKind};
use crate::units_lex::{
    COMPARATIVES, COUNT_NOUNS, CURRENCY_WORDS, CurrencyUnit, DATA_WORDS, MAGNITUDE_WORDS,
    PERCENT_WORDS, TIME_WORDS, contains, currency_symbol, lookup,
};

/// The quantity that starts at the first token, and how many tokens it is.
/// `None` when the tokens do not start with a quantity the grammar reads.
pub fn parse_quantity(tokens: &[Token]) -> Option<(usize, Quantity)> {
    let core = core(tokens)?;
    let (mut used, mut value, mut unit) = (core.used, core.value, core.unit);
    let next_word = |at: usize| {
        tokens
            .get(at)
            .filter(|t| t.kind == TokenKind::Word)
            .map(|t| t.text.as_str())
    };

    // 2. One magnitude word, when the core has none of its own: "2 million", "1.5 billion".
    if !core.scaled
        && let Some(m) = next_word(used).and_then(|w| lookup(MAGNITUDE_WORDS, w))
    {
        value *= m;
        used += 1;
    }
    // 3. "grand": a thousand, and dollars unless a currency is set. "five grand".
    else if !core.scaled && next_word(used) == Some("grand") {
        value *= 1_000.0;
        used += 1;
        if !matches!(unit, Unit::Usd | Unit::Eur | Unit::Gbp | Unit::Inr) {
            unit = Unit::Usd;
        }
    }

    // 4. At most one unit, when the core's own sign gave none.
    if unit == Unit::None {
        if let Some((more, found)) = unit_after(tokens.get(used..).unwrap_or(&[])) {
            used += more;
            unit = found;
        }
    } else if let Some(c) = core.currency
        && next_word(used).and_then(|w| lookup(CURRENCY_WORDS, w)) == Some(c)
    {
        // "$500 dollars": a currency word after a symbol of the same currency.
        used += 1;
    }
    Some((used, quantity(value, unit)))
}

/// What the first stage read.
struct Core {
    used: usize,
    value: f64,
    unit: Unit,
    /// The currency of a leading symbol, if any.
    currency: Option<CurrencyUnit>,
    /// The core already holds a magnitude: a suffix (`10k`) or a magnitude word ("two thousand").
    scaled: bool,
    /// Digits with no sign or suffix after them and no decimal point: split groups may follow.
    plain: bool,
}

/// 1. The core: a `Digits` token, or a spelled cardinal and an optional "point" fraction.
fn core(tokens: &[Token]) -> Option<Core> {
    let first = tokens.first()?;
    match first.kind {
        TokenKind::Digits => {
            let mut d = digits(&first.text)?;
            // V2's split groups (D-42) follow a plain core only: `$12` `,000`.
            if d.plain {
                (d.used, d.value) = with_split_groups(tokens, first, d.value);
            }
            Some(d)
        }
        TokenKind::Word => {
            let mut words: Vec<&str> = tokens
                .iter()
                .take(MAX_CARDINAL_TOKENS)
                .take_while(|t| t.kind == TokenKind::Word)
                .map(|t| t.text.as_str())
                .collect();
            // "an" reads like "a" before "hundred" or a magnitude ("an extra" is no number).
            if let Some(w) = words.first_mut()
                && *w == "an"
            {
                *w = "a";
            }
            let (used, whole) = cardinal(&words)?;
            let scaled = words
                .iter()
                .take(used)
                .any(|w| lookup(MAGNITUDE_WORDS, w).is_some());
            let fraction = words.get(used..).and_then(point_fraction);
            let (used, value) = match fraction {
                Some((more, f)) => (used + more, whole as f64 + f),
                None => (used, whole as f64),
            };
            Some(Core {
                used,
                value,
                unit: Unit::None,
                currency: None,
                scaled,
                plain: false,
            })
        }
        TokenKind::Punct => None,
    }
}

/// A `Digits` token: `$10,000`, `40%`, `10k`, `$1.5m`, `3x`, `€500`.
fn digits(text: &str) -> Option<Core> {
    let mut chars = text.chars();
    let currency = chars.next().and_then(currency_symbol);
    let text = if currency.is_some() {
        chars.as_str()
    } else {
        text
    };
    let (number, sign) = match text.chars().last() {
        Some(c @ ('%' | 'k' | 'm' | 'b' | 'x')) => {
            (text.get(..text.len() - c.len_utf8())?, Some(c))
        }
        _ => (text, None),
    };
    let base = parse_decimal(number)?;
    let (value, unit, scaled) = match (currency, sign) {
        // A currency and a share, or a currency and a multiplier, are no quantity.
        (Some(_), Some('%' | 'x')) => return None,
        (_, Some('%')) => (base, Unit::Percent, false),
        (_, Some('x')) => (base, Unit::Times, false),
        (c, Some('k')) => (base * 1e3, c.map_or(Unit::None, Unit::from), true),
        (c, Some('m')) => (base * 1e6, c.map_or(Unit::None, Unit::from), true),
        (c, Some('b')) => (base * 1e9, c.map_or(Unit::None, Unit::from), true),
        (c, _) => (base, c.map_or(Unit::None, Unit::from), false),
    };
    let plain = sign.is_none() && !number.contains('.');
    Some(Core {
        used: 1,
        value,
        unit,
        currency,
        scaled,
        plain,
    })
}

/// The unit in the one or two tokens after a number, and how many it uses.
fn unit_after(tokens: &[Token]) -> Option<(usize, Unit)> {
    let word = |at: usize| {
        tokens
            .get(at)
            .filter(|t| t.kind == TokenKind::Word)
            .map(|t| t.text.as_str())
    };
    let first = word(0)?;
    if contains(PERCENT_WORDS, first) {
        return Some((1, Unit::Percent));
    }
    if first == "per" && word(1) == Some("cent") {
        return Some((2, Unit::Percent));
    }
    if let Some(c) = lookup(CURRENCY_WORDS, first) {
        return Some((1, Unit::from(c)));
    }
    // "ten times faster": "times" only before a comparative, which is not consumed (D-102).
    if first == "times" {
        return word(1)
            .filter(|w| contains(COMPARATIVES, w))
            .map(|_| (1, Unit::Times));
    }
    if let Some(t) = lookup(TIME_WORDS, first) {
        return Some((1, Unit::from(t)));
    }
    if let Some(d) = lookup(DATA_WORDS, first) {
        return Some((1, Unit::from(d)));
    }
    contains(COUNT_NOUNS, first).then(|| {
        (
            1,
            Unit::Count {
                noun: first.to_owned(),
            },
        )
    })
}

fn quantity(value: f64, unit: Unit) -> Quantity {
    Quantity {
        value,
        display: format_quantity(value, &unit),
        unit,
    }
}

/// Digits with thousands separators and at most one decimal point: `10,000`,
/// `3.5`. A separator must have groups of three digits after it.
pub(crate) fn parse_decimal(text: &str) -> Option<f64> {
    let (whole, fraction) = match text.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (text, None),
    };
    let all_digits = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
    if fraction.is_some_and(|fraction| !all_digits(fraction)) {
        return None;
    }
    let mut groups = whole.split(',');
    let head = groups.next()?;
    let mut grouped = false;
    for group in groups {
        if group.len() != 3 || !all_digits(group) {
            return None;
        }
        grouped = true;
    }
    if !all_digits(head) || (grouped && head.len() > 3) {
        return None;
    }
    text.replace(',', "").parse().ok()
}

/// A recognizer may write `$12,000` as two words, `$12` and `,000`: it starts
/// a new word at the separator. A word that begins with a comma and exactly
/// three digits, straight after a whole number, is the next group of that
/// number (D-42). Nothing else that is written begins a word that way.
/// Returns the tokens used and the value.
fn with_split_groups(tokens: &[Token], first: &Token, value: f64) -> (usize, f64) {
    let (mut used, mut value, mut word) = (1usize, value, first.word);
    while let (Some(comma), Some(group)) = (tokens.get(used), tokens.get(used.saturating_add(1))) {
        let starts_next_word =
            comma.word == group.word && comma.word.get() == word.get().saturating_add(1);
        let is_group = comma.kind == TokenKind::Punct
            && comma.text == ","
            && group.kind == TokenKind::Digits
            && group.text.len() == 3
            && group.text.bytes().all(|b| b.is_ascii_digit());
        let Some(digits) = group
            .text
            .parse::<f64>()
            .ok()
            .filter(|_| starts_next_word && is_group)
        else {
            break;
        };
        value = value * 1_000.0 + digits;
        used = used.saturating_add(2);
        word = group.word;
    }
    (used, value)
}

/// A number as a caption shows it (v3implementation §8.5). A negative value
/// is `-` and the form of its size; a value that is not finite is `""`.
pub fn format_quantity(value: f64, unit: &Unit) -> String {
    if !value.is_finite() {
        return String::new();
    }
    let form = positive_form(value.abs(), unit);
    if value < 0.0 && plain(value.abs()) != "0" {
        format!("-{form}")
    } else {
        form
    }
}

fn positive_form(v: f64, unit: &Unit) -> String {
    let none = || none_form(v);
    match unit {
        Unit::Usd => money('$', v),
        Unit::Eur => money('€', v),
        Unit::Gbp => money('£', v),
        Unit::Inr => money('₹', v),
        Unit::Percent => format!("{}%", none()),
        Unit::Times => format!("{}x", none()),
        Unit::Milliseconds => format!("{} ms", none()),
        Unit::Seconds => timed(v, &none(), "second"),
        Unit::Minutes => timed(v, &none(), "minute"),
        Unit::Hours => timed(v, &none(), "hour"),
        Unit::Days => timed(v, &none(), "day"),
        Unit::Weeks => timed(v, &none(), "week"),
        Unit::Months => timed(v, &none(), "month"),
        Unit::Years => timed(v, &none(), "year"),
        Unit::Bytes => timed(v, &none(), "byte"),
        Unit::Kilobytes => format!("{} KB", none()),
        Unit::Megabytes => format!("{} MB", none()),
        Unit::Gigabytes => format!("{} GB", none()),
        Unit::Count { noun } => format!("{} {noun}", none()),
        Unit::None => none(),
    }
}

/// A currency: `$950`, `$10k`, `$1,500`, `$2m`.
fn money(symbol: char, v: f64) -> String {
    match (scaled(v), whole_thousands(v)) {
        (Some((n, _, letter)), _) => format!("{symbol}{n}{letter}"),
        (None, Some(k)) => format!("{symbol}{k}k"),
        (None, None) => format!("{symbol}{}", plain(v)),
    }
}

/// `1 hour`, `90 minutes`: the singular for exactly 1.
fn timed(v: f64, none: &str, word: &str) -> String {
    if v == 1.0 {
        format!("1 {word}")
    } else {
        format!("{none} {word}s")
    }
}

/// Without a unit: grouped by thousands under a million, else `2 million`, `1.5 billion`.
fn none_form(v: f64) -> String {
    match scaled(v) {
        Some((n, word, _)) => format!("{n} {word}"),
        None => plain(v),
    }
}

/// A value of a million or more in millions, billions or trillions, with up to
/// two decimals: the number, the word and the letter. A value that rounds to
/// 1,000 of one scale is written in the next ("1 billion", not "1,000 million").
fn scaled(v: f64) -> Option<(String, &'static str, char)> {
    const SCALES: [(f64, &str, char); 3] = [
        (1e6, "million", 'm'),
        (1e9, "billion", 'b'),
        (1e12, "trillion", 't'),
    ];
    let mut at = SCALES.iter().rposition(|(scale, ..)| v >= *scale)?;
    if SCALES
        .get(at)
        .is_some_and(|(scale, ..)| plain(v / scale) == "1,000")
        && at + 1 < SCALES.len()
    {
        at += 1;
    }
    let (scale, word, letter) = SCALES.get(at)?;
    Some((plain(v / scale), word, *letter))
}

/// `Some(10)` for 10,000: a whole number of thousands from 1 to 999.
fn whole_thousands(value: f64) -> Option<u32> {
    let whole = (1_000.0..=999_000.0).contains(&value) && value % 1_000.0 == 0.0;
    whole.then_some((value / 1_000.0) as u32)
}

/// Grouped by thousands, with up to two decimals and no trailing zero.
fn plain(value: f64) -> String {
    if !value.is_finite() {
        return "0".to_owned();
    }
    let rounded = format!("{:.2}", value.abs());
    let (whole, fraction) = rounded.split_once('.').unwrap_or((&rounded, ""));
    let fraction = fraction.trim_end_matches('0');
    let is_zero = fraction.is_empty() && whole.bytes().all(|b| b == b'0');

    let mut out = String::new();
    if value < 0.0 && !is_zero {
        out.push('-');
    }
    let mut left = whole.len();
    for digit in whole.chars() {
        out.push(digit);
        left = left.saturating_sub(1);
        if left > 0 && left.is_multiple_of(3) {
            out.push(',');
        }
    }
    if !fraction.is_empty() {
        out.push('.');
        out.push_str(fraction);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenize::tests::tokens_of;

    /// The quantity at the start of these words: tokens used, value, unit, display.
    fn read(texts: &[&str]) -> Option<(usize, f64, Unit, String)> {
        parse_quantity(&tokens_of(texts)).map(|(used, q)| (used, q.value, q.unit, q.display))
    }

    fn some(
        used: usize,
        value: f64,
        unit: Unit,
        display: &str,
    ) -> Option<(usize, f64, Unit, String)> {
        Some((used, value, unit, display.to_owned()))
    }

    #[test]
    fn forty_percent_is_one_digits_token_with_its_sign() {
        let tokens = tokens_of(&["40%"]);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Digits);
        assert_eq!(read(&["40%"]), some(1, 40.0, Unit::Percent, "40%"));

        // Digits without a sign, with separators, with a decimal point.
        assert_eq!(read(&["10,000"]), some(1, 10_000.0, Unit::None, "10,000"));
        assert_eq!(read(&["3.5"]), some(1, 3.5, Unit::None, "3.5"));
        assert_eq!(
            read(&["1234567"]),
            some(1, 1_234_567.0, Unit::None, "1.23 million")
        );
        // Not numbers: a separator in the wrong place, two points, both signs.
        for not_a_number in ["1,00", "10,0000", "1.2.3", "$40%"] {
            assert_eq!(read(&[not_a_number]), None, "{not_a_number}");
        }
        for not_a_decimal in [
            "", ",500", "5,", ".5", "5.", "1,2345", "1234,567", "1e3", "-5",
        ] {
            assert_eq!(parse_decimal(not_a_decimal), None, "{not_a_decimal}");
        }
        assert_eq!(parse_decimal("999,999.25"), Some(999_999.25));
    }

    #[test]
    fn ten_thousand_in_words_is_two_tokens_and_10_000() {
        assert_eq!(
            read(&["ten", "thousand", "people"]),
            some(3, 10_000.0, count("people"), "10,000 people")
        );
    }

    #[test]
    fn two_hundred_and_fifty_is_four_tokens() {
        assert_eq!(
            read(&["two", "hundred", "and", "fifty"]),
            some(4, 250.0, Unit::None, "250")
        );
        // An "and" that no number follows is left where it is.
        assert_eq!(
            read(&["two", "hundred", "and", "then"]),
            some(2, 200.0, Unit::None, "200")
        );
        assert_eq!(
            read(&["one", "thousand", "and", "so"]),
            some(2, 1_000.0, Unit::None, "1,000")
        );
    }

    #[test]
    fn three_is_3_with_no_unit() {
        assert_eq!(read(&["three"]), some(1, 3.0, Unit::None, "3"));

        // The whole range, built from every kind of word.
        let cases: [(&[&str], usize, f64); 10] = [
            (&["zero"], 1, 0.0),
            (&["nineteen"], 1, 19.0),
            (&["forty", "two"], 2, 42.0),
            (&["forty-two"], 1, 42.0),
            (&["nine", "hundred", "ninety", "nine"], 4, 999.0),
            (&["fifteen", "hundred"], 2, 1_500.0),
            (&["one", "thousand", "and", "five"], 4, 1_005.0),
            (&["twelve", "thousand"], 2, 12_000.0),
            (
                &["three", "hundred", "thousand", "two", "hundred"],
                5,
                300_200.0,
            ),
            (
                &[
                    "nine", "hundred", "and", "ninety", "nine", "thousand", "nine", "hundred",
                    "and", "ninety", "nine",
                ],
                11,
                999_999.0,
            ),
        ];
        for (texts, used, value) in cases {
            let found = read(texts).map(|(used, value, ..)| (used, value));
            assert_eq!(found, Some((used, value)), "{texts:?}");
        }
        // Words that are not a number, and words a number needs something before.
        let not_numbers: [&[&str]; 4] = [&["hundred"], &["thousand"], &["and", "five"], &["hello"]];
        for texts in not_numbers {
            assert_eq!(read(texts), None, "{texts:?}");
        }
        // Read out one after the other, each word is a number of its own.
        assert_eq!(
            read(&["one", "two", "three"]),
            some(1, 1.0, Unit::None, "1")
        );
    }

    #[test]
    fn what_v3_reads_is_read_and_nothing_is_guessed_at() {
        // V2 read "ten" and left "million" for V3 (v3implementation §8.6).
        assert_eq!(
            read(&["ten", "million"]),
            some(2, 10_000_000.0, Unit::None, "10 million")
        );
        assert_eq!(
            read(&["three", "point", "five"]),
            some(3, 3.5, Unit::None, "3.5")
        );
        assert_eq!(read(&["2k"]), some(1, 2_000.0, Unit::None, "2,000"));
        assert_eq!(read(&["percent"]), None);
        assert_eq!(read(&["."]), None);
        assert_eq!(read(&[]), None);
    }

    #[test]
    fn a_number_the_recognizer_split_at_its_separator_is_one_number() {
        // The reference clip's amount, as the recognizer wrote it: two words.
        assert_eq!(read(&["$12", ",000"]), some(3, 12_000.0, Unit::Usd, "$12k"));
        assert_eq!(
            read(&["$12", ",000", "in", "sales."]),
            some(3, 12_000.0, Unit::Usd, "$12k")
        );
        assert_eq!(
            read(&["3", ",000", ",000."]),
            some(5, 3_000_000.0, Unit::None, "3 million")
        );
        assert_eq!(
            read(&["1,250", ",500"]),
            some(3, 1_250_500.0, Unit::None, "1.25 million")
        );

        // A comma that ends a word is a comma: the number after it is another number.
        assert_eq!(read(&["12,", "000"]), some(1, 12.0, Unit::None, "12"));
        // Not a group of three digits, a number that had ended, a group in a later word.
        assert_eq!(read(&["$12", ",00"]), some(1, 12.0, Unit::Usd, "$12"));
        assert_eq!(read(&["$12", ",0000"]), some(1, 12.0, Unit::Usd, "$12"));
        assert_eq!(read(&["1.5", ",000"]), some(1, 1.5, Unit::None, "1.5"));
        assert_eq!(read(&["40%", ",000"]), some(1, 40.0, Unit::Percent, "40%"));
        assert_eq!(read(&["$12.", ",000"]), some(1, 12.0, Unit::Usd, "$12"));
        assert_eq!(
            read(&["$12", "and", ",000"]),
            some(1, 12.0, Unit::Usd, "$12")
        );
    }

    #[test]
    fn dollars_are_short_only_in_whole_thousands() {
        assert_eq!(read(&["$10,000"]), some(1, 10_000.0, Unit::Usd, "$10k"));
        assert_eq!(read(&["$1,500"]), some(1, 1_500.0, Unit::Usd, "$1,500"));
        assert_eq!(read(&["$950"]), some(1, 950.0, Unit::Usd, "$950"));

        assert_eq!(format_quantity(999_000.0, &Unit::Usd), "$999k");
        assert_eq!(format_quantity(1_000_000.0, &Unit::Usd), "$1m");
        assert_eq!(format_quantity(12.5, &Unit::Usd), "$12.5");
        // Up to two decimals, without a zero at the end.
        assert_eq!(format_quantity(1_234.5, &Unit::None), "1,234.5");
        assert_eq!(format_quantity(0.126, &Unit::None), "0.13");
        assert_eq!(format_quantity(2.999, &Unit::None), "3");
        assert_eq!(format_quantity(7.10, &Unit::Percent), "7.1%");
        assert_eq!(format_quantity(-1_500.0, &Unit::None), "-1,500");
        assert_eq!(format_quantity(-0.001, &Unit::None), "0");
        assert_eq!(format_quantity(f64::NAN, &Unit::None), "");
        assert_eq!(format_quantity(800.0, &Unit::Milliseconds), "800 ms");
    }

    /// v3implementation §8.7: a negative value and a value that is not finite.
    #[test]
    fn a_negative_value_and_nan() {
        assert_eq!(format_quantity(-5.0, &Unit::None), "-5");
        assert_eq!(format_quantity(f64::NAN, &Unit::None), "");
        assert_eq!(format_quantity(f64::INFINITY, &Unit::Usd), "");
    }

    /// The four stages of §8.3 and the display table of §8.5, a row of each.
    #[test]
    fn the_four_stages_and_the_display_table() {
        assert_eq!(read(&["$1.5M"]), some(1, 1_500_000.0, Unit::Usd, "$1.5m"));
        assert_eq!(read(&["€500"]), some(1, 500.0, Unit::Eur, "€500"));
        assert_eq!(read(&["3x"]), some(1, 3.0, Unit::Times, "3x"));
        assert_eq!(read(&["$3x"]), None);
        assert_eq!(
            read(&["2", "million"]),
            some(2, 2_000_000.0, Unit::None, "2 million")
        );
        assert_eq!(
            read(&["1.5", "billion"]),
            some(2, 1.5e9, Unit::None, "1.5 billion")
        );
        assert_eq!(
            read(&["three", "point", "five", "million"]),
            some(4, 3_500_000.0, Unit::None, "3.5 million")
        );
        assert_eq!(read(&["five", "grand"]), some(2, 5_000.0, Unit::Usd, "$5k"));
        assert_eq!(
            read(&["a", "thousand", "people"]),
            some(3, 1_000.0, count("people"), "1,000 people")
        );
        assert_eq!(
            read(&["40", "percent"]),
            some(2, 40.0, Unit::Percent, "40%")
        );
        assert_eq!(
            read(&["12", "per", "cent"]),
            some(3, 12.0, Unit::Percent, "12%")
        );
        assert_eq!(
            read(&["fifty", "thousand", "rupees"]),
            some(3, 50_000.0, Unit::Inr, "₹50k")
        );
        assert_eq!(
            read(&["$500", "dollars"]),
            some(2, 500.0, Unit::Usd, "$500")
        );
        assert_eq!(
            read(&["ten", "times", "faster"]),
            some(2, 10.0, Unit::Times, "10x")
        );
        assert_eq!(read(&["three", "times"]), some(1, 3.0, Unit::None, "3"));
        assert_eq!(read(&["ten", "pounds"]), some(1, 10.0, Unit::None, "10"));
        assert_eq!(read(&["1", "hour"]), some(2, 1.0, Unit::Hours, "1 hour"));
        assert_eq!(
            read(&["90", "minutes"]),
            some(2, 90.0, Unit::Minutes, "90 minutes")
        );
        assert_eq!(
            read(&["250", "gigabytes"]),
            some(2, 250.0, Unit::Gigabytes, "250 GB")
        );
        assert_eq!(
            read(&["512", "bytes"]),
            some(2, 512.0, Unit::Bytes, "512 bytes")
        );
        assert_eq!(
            read(&["10,000", "subscribers"]),
            some(2, 10_000.0, count("subscribers"), "10,000 subscribers")
        );
        for not_read in [
            &["₹2,00,000"][..],
            &["10:30"],
            &["5-10"],
            &["half"],
            &["dozen"],
        ] {
            assert_eq!(read(not_read), None, "{not_read:?}");
        }
        assert_eq!(format_quantity(999_999_999.0, &Unit::None), "1 billion");
        assert_eq!(format_quantity(1_250_000.0, &Unit::Usd), "$1.25m");
        assert_eq!(format_quantity(3e12, &Unit::Eur), "€3t");
        assert_eq!(format_quantity(1.5, &Unit::Days), "1.5 days");
        assert_eq!(format_quantity(1.0, &Unit::Bytes), "1 byte");
    }

    fn count(noun: &str) -> Unit {
        Unit::Count {
            noun: noun.to_owned(),
        }
    }
}
