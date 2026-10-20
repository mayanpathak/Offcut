//! The numbers that are not what they look like, and the events a hidden
//! word removes (TS §17.4 "Exclusions", §17.6; section 9.7). A rule sets the
//! candidate's score to 0, and the threshold then drops it.

use std::collections::BTreeMap;

use offcut_text::{Token, TokenKind};
use offcut_types::{EventKind, Transcript, Unit, WordIdx, WordRange};

use crate::config::{
    BARE_NUMBER_MIN, CLOCK_WORDS, PHONE_DIGITS_MIN, VERSION_AFTER, VERSION_CONTEXT, VERSION_DOTS,
    YEAR_FIRST, YEAR_LAST,
};
use crate::list::markers;
use crate::number::{Candidate, Found, quantities};

/// Zeroes every candidate a rule excludes. X1-X5 look at the quantities of a
/// NumberReveal or a FromTo, X6 at a NumberReveal's, X7 at every kind. A
/// list's announcer count is never checked: its value is 2 to 7 by design.
/// This is the one file that reads `edits` itself (X7).
pub fn apply(
    cands: &mut [Candidate],
    toks: &[Token],
    t: &Transcript,
    edits: &BTreeMap<WordIdx, String>,
) {
    let found = quantities(toks, t);
    let marked: Vec<WordRange> = markers(toks, t).iter().map(|m| m.words).collect();
    for cand in cands.iter_mut() {
        let (numbers, x6) = match cand.kind {
            EventKind::NumberReveal => (true, true),
            EventKind::FromTo => (true, false),
            EventKind::ListReveal | EventKind::KeywordPop => (false, false),
        };
        let excluded = cand.parts.iter().any(|part| {
            let quantity = found.iter().find(|f| f.words == *part);
            (numbers && quantity.is_some_and(|f| not_an_amount(toks, f)))
                || (x6 && marked.contains(part))
        });
        if excluded || hidden_inside(edits, cand.anchors) {
            cand.score = 0.0;
        }
    }
}

/// X1-X5, on the quantity's own unit, before a FromTo's inheritance.
fn not_an_amount(toks: &[Token], f: &Found) -> bool {
    let own = toks.get(f.tokens.clone()).unwrap_or(&[]);
    let bare = f.quantity.unit == Unit::None;
    let value = f.quantity.value;
    // One `Digits` token of digits only: no sign, separator or point.
    let plain = match own {
        [one] if one.kind == TokenKind::Digits && one.text.bytes().all(|b| b.is_ascii_digit()) => {
            Some(one.text.len())
        }
        _ => None,
    };
    let before = f.tokens.start.checked_sub(1).and_then(|at| toks.get(at));
    let after = toks.get(f.tokens.end);

    let year = bare && plain.is_some() && (YEAR_FIRST..=YEAR_LAST).contains(&value);
    let version = word_in(before, VERSION_CONTEXT)
        || word_in(after, VERSION_AFTER)
        || own
            .iter()
            .map(|token| token.text.matches('.').count())
            .sum::<usize>()
            >= VERSION_DOTS;
    let clock = word_in(after, CLOCK_WORDS);
    let phone = bare && plain.is_some_and(|digits| digits >= PHONE_DIGITS_MIN);
    let small = bare && value.abs() < BARE_NUMBER_MIN;
    year || version || clock || phone || small
}

fn word_in(token: Option<&Token>, words: &[&str]) -> bool {
    token.is_some_and(|token| {
        token.kind == TokenKind::Word && words.binary_search(&token.text.as_str()).is_ok()
    })
}

/// X7: a word inside the anchors was hidden, an edit to nothing (TS §17.6).
fn hidden_inside(edits: &BTreeMap<WordIdx, String>, anchors: WordRange) -> bool {
    edits
        .range(anchors.start..anchors.end)
        .any(|(_, text)| text.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use offcut_text::{RawWord, normalize_transcript, tokenize};
    use offcut_types::{Prosody, WordProsody};

    use super::*;
    use crate::config::DetectorConfig;
    use crate::{from_to, list, number};

    /// Words 300 ms long and 100 ms apart, all certain.
    fn transcript(text: &str) -> Transcript {
        let raw = (0u32..)
            .zip(text.split_whitespace())
            .map(|(i, word)| RawWord {
                text: word.to_owned(),
                start_ms: i * 400,
                end_ms: i * 400 + 300,
                confidence: 1.0,
            })
            .collect();
        normalize_transcript(raw, "test")
    }

    /// The candidates of the three finders that have parts, before and after
    /// the exclusions, as `(kind, score before, score after)`.
    fn excluded(text: &str, hide: &[u32]) -> Vec<(EventKind, f32, f32)> {
        let t = transcript(text);
        let edits: BTreeMap<WordIdx, String> = hide
            .iter()
            .map(|word| (WordIdx::new(*word), String::new()))
            .collect();
        let toks = tokenize(&t.words, &edits);
        let p = Prosody {
            per_word: vec![
                WordProsody {
                    energy_z: 0.0,
                    pitch_z: 0.0,
                };
                t.words.len()
            ],
        };
        let cfg = DetectorConfig::default();
        let mut cands = from_to::find(&toks, &t, &p, &cfg);
        cands.extend(list::find(&toks, &t, &p, &cfg));
        cands.extend(number::find(&toks, &t, &p, &cfg));
        let before: Vec<f32> = cands.iter().map(|c| c.score).collect();
        apply(&mut cands, &toks, &t, &edits);
        cands
            .iter()
            .zip(before)
            .map(|(c, before)| (c.kind, before, c.score))
            .collect()
    }

    /// The NumberReveal candidates' scores after the exclusions.
    fn numbers(text: &str) -> Vec<f32> {
        excluded(text, &[])
            .into_iter()
            .filter(|(kind, ..)| *kind == EventKind::NumberReveal)
            .map(|(.., after)| after)
            .collect()
    }

    #[test]
    fn x1_a_bare_year_in_plain_digits() {
        assert_eq!(numbers("in 2024 we moved"), [0.0]);
        assert_eq!(numbers("in 1899 we moved"), [0.7]);
        // With a unit or a separator it is an amount.
        assert_eq!(numbers("we paid $2024 then"), [1.0]);
        assert_eq!(numbers("we had 2,024 then"), [0.7]);
    }

    #[test]
    fn x2_a_version_or_a_product() {
        assert_eq!(numbers("Upgrade to Python 3.12 now."), [0.0]);
        assert_eq!(numbers("The iPhone 15 Pro is here."), [0.0]);
        assert_eq!(numbers("It is version 5000 now."), [0.0]);
        assert_eq!(numbers("It hit 5000 today."), [0.7]);
    }

    #[test]
    fn x3_a_time_of_day() {
        assert_eq!(numbers("Meet me at 11 pm."), [0.0]);
        // "a.m." is the word `a.m` and a full stop (open item 15).
        assert_eq!(numbers("Meet me at 11 a.m. today."), [0.0]);
        assert_eq!(numbers("Meet me at 11 o'clock."), [0.0]);
        assert_eq!(numbers("Meet 11 people."), [0.5]);
    }

    #[test]
    fn x4_a_phone_like_run_of_digits() {
        assert_eq!(numbers("Call 5551234 today."), [0.0]);
        assert_eq!(numbers("Call 555123 today."), [0.7]);
        assert_eq!(numbers("We made $5551234 today."), [1.0]);
    }

    #[test]
    fn x5_a_bare_small_number_before_inheritance() {
        assert_eq!(numbers("I ate 3 apples today."), [0.0]);
        // A count noun is a unit: "3 users" is not bare (it scores 0.50 anyway).
        assert_eq!(numbers("We had 3 users today."), [0.5]);
        assert_eq!(numbers("It costs $3 now."), [0.8]);
        // "3" is bare until it takes "people": the FromTo goes.
        let found = excluded("from 3 to 5 people", &[]);
        let pair = found.iter().find(|(kind, ..)| *kind == EventKind::FromTo);
        assert!(pair.is_some_and(|(_, before, after)| *before > 0.85 && *after == 0.0));
        // The years of a FromTo (TS §17.6).
        let found = excluded("from 2019 to 2024", &[]);
        let pair = found.iter().find(|(kind, ..)| *kind == EventKind::FromTo);
        assert!(pair.is_some_and(|(_, before, after)| *before == 1.0 && *after == 0.0));
    }

    #[test]
    fn x6_a_number_that_is_a_list_marker_and_the_announcer_is_exempt() {
        let found = excluded("I have 3 tips. 1, sleep. 2, eat. 3, ship.", &[]);
        let list = found
            .iter()
            .find(|(kind, ..)| *kind == EventKind::ListReveal);
        assert!(list.is_some_and(|(_, before, after)| before == after && *after > 0.85));
        // "3" of "3 tips" and the three markers are bare and small (X5); the
        // markers are also markers (X6). A marker is a bare number from 1 to
        // 7, so X5 always catches what X6 does; X6 is kept as TS §17.4 lists it.
        let numbers: Vec<f32> = found
            .iter()
            .filter(|(kind, ..)| *kind == EventKind::NumberReveal)
            .map(|(.., after)| *after)
            .collect();
        assert_eq!(numbers, [0.0; 4]);
    }

    #[test]
    fn x7_a_hidden_word_inside_a_list_item_removes_the_list() {
        let text = "There are three reasons it worked. First the captions write themselves. \
                    Second the numbers animate automatically. Third everything runs on my laptop.";
        let list = |hide: &[u32]| {
            excluded(text, hide)
                .into_iter()
                .find(|(kind, ..)| *kind == EventKind::ListReveal)
                .map(|(.., after)| after)
        };
        assert_eq!(list(&[]), Some(1.0));
        // Word 13, "numbers", inside the second item.
        assert_eq!(list(&[13]), Some(0.0));
        // A hidden word outside the list's words changes nothing.
        assert_eq!(list(&[0]), Some(1.0));
    }
}
