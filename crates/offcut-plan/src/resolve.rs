//! Numbers from words (v4implementation D-121, INV-22): a plan points at the
//! words that say a number, and the number is read here from the effective
//! transcript, with the user's word edits. Nothing else in the plan crate
//! parses a number, and the scene never does.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use offcut_text::{Token, TokenKind, format_quantity, parse_quantity, tokenize};
use offcut_types::{Quantity, Transcript, Unit, WordIdx, WordRange};

/// The tokens of the transcript as the user sees it: each edited word's new
/// text in place of what was transcribed; a hidden word gives none.
pub(crate) fn effective_tokens(t: &Transcript, edits: &BTreeMap<WordIdx, String>) -> Vec<Token> {
    tokenize(&t.words, edits)
}

/// The quantity the words of `words` say, if they say exactly one: every word
/// of the range has a token, the quantity starts at the first token, and it
/// takes every token of the range but trailing punctuation. "$12k next"
/// pointed at three words gives nothing: the range must be the number.
pub(crate) fn quantity_at(tokens: &[Token], words: WordRange) -> Option<Quantity> {
    if words.start >= words.end {
        return None;
    }
    let first = tokens.iter().position(|token| token.word >= words.start)?;
    let rest = tokens.get(first..)?;
    let len = rest
        .iter()
        .take_while(|token| token.word < words.end)
        .count();
    let inside = rest.get(..len)?;
    // A word of the range without a token was hidden by an edit.
    let every_word_said = (words.start.get()..words.end.get())
        .all(|word| inside.iter().any(|token| token.word.get() == word));
    if !every_word_said {
        return None;
    }
    let (used, quantity) = parse_quantity(inside)?;
    let left = inside.get(used..)?;
    left.iter()
        .all(|token| token.kind == TokenKind::Punct)
        .then_some(quantity)
}

/// The quantity with `unit`, its display written again for it.
fn carried(quantity: Quantity, unit: &Unit) -> Quantity {
    Quantity {
        display: format_quantity(quantity.value, unit),
        value: quantity.value,
        unit: unit.clone(),
    }
}

/// A change's two ends with units that agree (D-85): a bare number takes the
/// other end's unit, and its display is written again. Two units, or none,
/// stay as they are.
pub(crate) fn inherit(from: Quantity, to: Quantity) -> (Quantity, Quantity) {
    if from.unit == Unit::None && to.unit != Unit::None {
        let unit = to.unit.clone();
        (carried(from, &unit), to)
    } else if to.unit == Unit::None && from.unit != Unit::None {
        let unit = from.unit.clone();
        (from, carried(to, &unit))
    } else {
        (from, to)
    }
}

/// The runs of ASCII digits in `text`.
fn digit_runs(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_ascii_digit())
        .filter(|run| !run.is_empty())
}

/// Every digit run of the label is a digit run of a token of the effective
/// transcript (TS §40.4 V-4): a plan cannot write a number nobody said.
pub(crate) fn label_digits_ok(label: &str, tokens: &[Token]) -> bool {
    let said: BTreeSet<&str> = tokens
        .iter()
        .flat_map(|token| digit_runs(&token.text))
        .collect();
    digit_runs(label).all(|run| said.contains(run))
}

#[cfg(test)]
mod tests {
    use offcut_text::{RawWord, normalize_transcript};

    use super::*;

    fn transcript(texts: &[&str]) -> Transcript {
        let words = (0u32..)
            .zip(texts)
            .map(|(i, text)| RawWord {
                text: (*text).to_owned(),
                start_ms: 500 + i * 400,
                end_ms: 800 + i * 400,
                confidence: 1.0,
            })
            .collect();
        normalize_transcript(words, "test")
    }

    fn range(start: u32, end: u32) -> WordRange {
        WordRange {
            start: WordIdx::new(start),
            end: WordIdx::new(end),
        }
    }

    #[test]
    fn spelled_words_are_one_number() {
        let t = transcript(&["about", "ten", "thousand", "people."]);
        let tokens = effective_tokens(&t, &BTreeMap::new());
        let q = quantity_at(&tokens, range(1, 3)).unwrap();
        assert_eq!(
            (q.value, &q.unit, q.display.as_str()),
            (10_000.0, &Unit::None, "10,000")
        );
        // A range with a word that is not part of the number says nothing.
        assert_eq!(quantity_at(&tokens, range(0, 3)), None);
        // An empty range says nothing.
        assert_eq!(quantity_at(&tokens, range(2, 2)), None);
    }

    #[test]
    fn an_edited_number_is_read_again_and_inherits_its_unit() {
        let t = transcript(&["from", "ninety", "to", "ten", "minutes."]);
        let edits = BTreeMap::from([(WordIdx::new(1), "nine".to_owned())]);
        let tokens = effective_tokens(&t, &edits);
        let from = quantity_at(&tokens, range(1, 2)).unwrap();
        let to = quantity_at(&tokens, range(3, 5)).unwrap();
        assert_eq!((from.value, &from.unit), (9.0, &Unit::None));
        let (from, to) = inherit(from, to);
        assert_eq!((from.value, &from.unit), (9.0, &Unit::Minutes));
        assert_eq!(from.display, format_quantity(9.0, &Unit::Minutes));
        assert_eq!((to.value, &to.unit), (10.0, &Unit::Minutes));
    }

    #[test]
    fn a_range_with_a_hidden_word_says_nothing() {
        let t = transcript(&["ten", "thousand", "dollars."]);
        let edits = BTreeMap::from([(WordIdx::new(1), String::new())]);
        let tokens = effective_tokens(&t, &edits);
        assert_eq!(quantity_at(&tokens, range(0, 3)), None);
        // Without the edit, the same range is the number.
        let whole = effective_tokens(&t, &BTreeMap::new());
        let q = quantity_at(&whole, range(0, 3)).unwrap();
        assert_eq!((q.value, &q.unit), (10_000.0, &Unit::Usd));
    }

    #[test]
    fn a_label_may_only_hold_digits_that_were_said() {
        let t = transcript(&["it", "is", "much", "faster", "now."]);
        let tokens = effective_tokens(&t, &BTreeMap::new());
        assert!(!label_digits_ok("3x faster", &tokens));
        assert!(label_digits_ok("much faster", &tokens));
        let said = transcript(&["it", "is", "3x", "faster."]);
        let tokens = effective_tokens(&said, &BTreeMap::new());
        assert!(label_digits_ok("3x faster", &tokens));
        assert!(!label_digits_ok("30x faster", &tokens));
    }

    #[test]
    fn inheritance_leaves_two_units_or_none_alone() {
        let q = |value: f64, unit: Unit| Quantity {
            display: format_quantity(value, &unit),
            value,
            unit,
        };
        let (a, b) = inherit(q(40.0, Unit::Percent), q(25.0, Unit::Percent));
        assert_eq!((a.unit, b.unit), (Unit::Percent, Unit::Percent));
        let (a, b) = inherit(q(4.0, Unit::None), q(9.0, Unit::None));
        assert_eq!((a.unit, b.unit), (Unit::None, Unit::None));
        let (a, b) = inherit(q(90.0, Unit::Minutes), q(10.0, Unit::None));
        assert_eq!(
            (a.display.as_str(), &b.unit),
            ("90 minutes", &Unit::Minutes)
        );
    }
}
