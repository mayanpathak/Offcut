//! FromTo candidates: two quantities of one sentence, the first changing into
//! the second (TS §17.4, D-85, D-106).

use offcut_text::{Token, TokenKind, format_quantity};
use offcut_types::{
    EventKind, EventParams, Prosody, Quantity, Span, Transcript, Unit, WordIdx, WordRange,
};

use crate::config::{CHANGE_VERBS, DetectorConfig, FROM, FROM_TO_VERB_REACH, TO};
use crate::number::{Candidate, Found, quantities};

/// For each quantity, its best partner later in the same sentence, if any.
pub fn find(toks: &[Token], t: &Transcript, _p: &Prosody, cfg: &DetectorConfig) -> Vec<Candidate> {
    let found = quantities(toks, t);
    let mut out = Vec::new();
    for (at, a) in found.iter().enumerate() {
        let mut best: Option<Candidate> = None;
        let later = found.get(at + 1..).unwrap_or(&[]);
        // The partners come nearest first, so of equal scores the nearest stays.
        for b in later.iter().take_while(|b| b.sentence == a.sentence) {
            if let Some(pair) = pair(toks, t, cfg, a, b)
                && best.as_ref().is_none_or(|kept| pair.score > kept.score)
            {
                best = Some(pair);
            }
        }
        out.extend(best);
    }
    out
}

/// `a` to `b`, when they can be one change: close enough, different values,
/// and units that agree once a bare number takes the other's (D-85).
fn pair(
    toks: &[Token],
    t: &Transcript,
    cfg: &DetectorConfig,
    a: &Found,
    b: &Found,
) -> Option<Candidate> {
    let between = toks.get(a.tokens.end..b.tokens.start)?;
    let gap = between
        .iter()
        .filter(|token| token.kind == TokenKind::Word)
        .count();
    if gap > cfg.from_to_max_gap_words || a.quantity.value == b.quantity.value {
        return None;
    }
    let (own_a, own_b) = (&a.quantity.unit, &b.quantity.unit);
    let (from_unit, to_unit) = match (own_a, own_b) {
        (Unit::None, unit) | (unit, Unit::None) => (unit, unit),
        (from, to) => (from, to),
    };
    if !compatible(from_unit, to_unit) {
        return None;
    }
    let inherits = own_a != own_b && (*own_a == Unit::None || *own_b == Unit::None);

    // The token right before a quantity, inside its sentence.
    let start = t.sentences.get(a.sentence.get() as usize)?.words.start;
    let before = |at: usize| {
        at.checked_sub(1)
            .and_then(|i| toks.get(i))
            .filter(|token| token.word >= start)
    };
    let to_before_b = says(before(b.tokens.start), TO);

    let mut score = cfg.from_to_base;
    if says(before(a.tokens.start), FROM) && to_before_b {
        score += cfg.from_to_frame_bonus;
    } else if to_before_b && changes(toks, start, a, between) {
        score += cfg.from_to_change_verb_bonus;
    }
    if inherits {
        score += cfg.from_to_inherit_bonus;
    } else if own_a == own_b {
        score += cfg.from_to_same_unit_bonus;
    }
    if gap <= cfg.from_to_close_gap_words {
        score += cfg.from_to_close_bonus;
    }

    let words = |w: WordRange| t.words.get(w.start.get() as usize..w.end.get() as usize);
    let (words_a, words_b) = (words(a.words)?, words(b.words)?);
    let heard = words_a
        .iter()
        .chain(words_b)
        .map(|word| word.confidence.get())
        .fold(1.0f32, f32::min);
    Some(Candidate {
        kind: EventKind::FromTo,
        anchors: WordRange {
            start: a.words.start,
            end: b.words.end,
        },
        span: Span::new(words_a.first()?.start_ms, words_b.last()?.end_ms)?,
        score: score.min(1.0) * heard,
        params: EventParams::FromTo {
            from: carried(&a.quantity, from_unit),
            to: carried(&b.quantity, to_unit),
            label: None,
        },
        parts: vec![a.words, b.words],
        sentence: a.sentence,
    })
}

fn says(token: Option<&Token>, word: &str) -> bool {
    token.is_some_and(|token| token.kind == TokenKind::Word && token.text == word)
}

/// A change verb among the few tokens before `a` in its sentence, or between
/// the two quantities: "dropped 40% to 25%", "was 9 hours and went down to 2".
fn changes(toks: &[Token], start: WordIdx, a: &Found, between: &[Token]) -> bool {
    let reach = a.tokens.start.saturating_sub(FROM_TO_VERB_REACH);
    let ahead = toks
        .get(reach..a.tokens.start)
        .unwrap_or(&[])
        .iter()
        .filter(|token| token.word >= start);
    ahead.chain(between).any(|token| {
        token.kind == TokenKind::Word && CHANGE_VERBS.binary_search(&token.text.as_str()).is_ok()
    })
}

#[derive(PartialEq)]
enum Group {
    Time,
    Data,
}

/// The same unit, or two of one group: a time and a time, a size and a size.
/// Two currencies differ; a count needs the same noun (D-85).
fn compatible(a: &Unit, b: &Unit) -> bool {
    a == b || group(a).is_some_and(|g| group(b) == Some(g))
}

fn group(unit: &Unit) -> Option<Group> {
    match unit {
        Unit::Milliseconds
        | Unit::Seconds
        | Unit::Minutes
        | Unit::Hours
        | Unit::Days
        | Unit::Weeks
        | Unit::Months
        | Unit::Years => Some(Group::Time),
        Unit::Bytes | Unit::Kilobytes | Unit::Megabytes | Unit::Gigabytes => Some(Group::Data),
        Unit::None
        | Unit::Usd
        | Unit::Eur
        | Unit::Gbp
        | Unit::Inr
        | Unit::Percent
        | Unit::Times
        | Unit::Count { .. } => None,
    }
}

/// The quantity with the unit it carries in the pair. One that took the
/// other's unit is written again with it: "40" becomes "40%" (D-106). This is
/// the one call of `format_quantity` outside `offcut-text`.
fn carried(quantity: &Quantity, unit: &Unit) -> Quantity {
    if quantity.unit == *unit {
        return quantity.clone();
    }
    Quantity {
        value: quantity.value,
        unit: unit.clone(),
        display: format_quantity(quantity.value, unit),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use offcut_text::{RawWord, normalize_transcript, tokenize};
    use offcut_types::{Confidence, WordProsody};

    use super::*;

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

    fn flat(t: &Transcript) -> Prosody {
        let neutral = WordProsody {
            energy_z: 0.0,
            pitch_z: 0.0,
        };
        Prosody {
            per_word: vec![neutral; t.words.len()],
        }
    }

    fn pairs_of(t: &Transcript) -> Vec<Candidate> {
        let toks = tokenize(&t.words, &BTreeMap::new());
        find(&toks, t, &flat(t), &DetectorConfig::default())
    }

    fn pairs(text: &str) -> Vec<Candidate> {
        pairs_of(&transcript(text))
    }

    fn ends(c: &Candidate) -> (&Quantity, &Quantity) {
        match &c.params {
            EventParams::FromTo { from, to, label } => {
                assert_eq!(*label, None);
                (from, to)
            }
            EventParams::NumberReveal { .. }
            | EventParams::ListReveal { .. }
            | EventParams::KeywordPop { .. } => panic!("{:?}", c.params),
        }
    }

    fn shown(c: &Candidate) -> (&str, &str) {
        let (from, to) = ends(c);
        (from.display.as_str(), to.display.as_str())
    }

    fn one(text: &str) -> Candidate {
        let found = pairs(text);
        assert_eq!(found.len(), 1, "{text}: {found:?}");
        found.into_iter().next().unwrap()
    }

    fn close(score: f32, expected: f32) -> bool {
        (score - expected).abs() < 1e-6
    }

    #[test]
    fn the_seven_rows_of_the_plan() {
        // A from frame, the same unit on both sides, close: 1.00.
        let c = one("It went from 90 minutes to 10 minutes.");
        assert_eq!(c.score, 1.0);
        assert_eq!(shown(&c), ("90 minutes", "10 minutes"));
        let c = one("from $2k to $20k");
        assert_eq!(c.score, 1.0);
        assert_eq!(shown(&c), ("$2k", "$20k"));
        // No frame: 0.75, under the threshold (TS §17.6).
        assert!(close(one("$2k to $20k").score, 0.75));
        // "40" takes the percent of "90%": 0.55 + 0.25 + 0.05 + 0.05.
        let c = one("revenue grew from 40 to 90%");
        assert!(close(c.score, 0.90));
        assert_eq!(shown(&c), ("40%", "90%"));
        assert_eq!(ends(&c).0.unit, Unit::Percent);
        // "to" comes before the first quantity, not the second: no frame.
        assert!(close(
            one("dropped to 5 minutes from 20 minutes").score,
            0.75
        ));
        // Years are a candidate here; the exclusions zero them (X1, Prompt 77).
        assert_eq!(one("from 2019 to 2024").score, 1.0);
        // "3" takes "people" (D-85); X5 zeroes it later, bare before inheritance.
        let c = one("from 3 to 5 people");
        assert!(close(c.score, 0.90));
        assert_eq!(shown(&c), ("3 people", "5 people"));
    }

    #[test]
    fn a_change_verb_before_or_between_is_the_weaker_frame() {
        let c = one("Costs dropped 40% to 25%.");
        assert!(close(c.score, 0.90));
        assert_eq!(shown(&c), ("40%", "25%"));
        let c = one("It was 9 hours and the rewrite brought it down to 2 hours.");
        assert!(close(c.score, 0.85));
        // A verb four tokens before is out of reach.
        assert!(close(one("It grew in the year 40% to 90%").score, 0.75));
    }

    #[test]
    fn units_must_agree_after_inheritance() {
        // A time and a time, without a bonus for the unit.
        let c = one("from 90 minutes to 2 hours");
        assert!(close(c.score, 0.85));
        assert_eq!(pairs("from 512 megabytes to 2 gigabytes").len(), 1);
        // Two currencies, a currency and a share, two different nouns: none.
        assert_eq!(pairs("from $500 to €900"), []);
        assert_eq!(pairs("from $500 to 90%"), []);
        assert_eq!(pairs("from 300 users to 900 customers"), []);
        assert_eq!(pairs("from 2 hours to 3 gigabytes"), []);
    }

    #[test]
    fn equal_values_far_apart_or_in_two_sentences_are_no_pair() {
        assert_eq!(pairs("from $20 to $20"), []);
        // Nine words between the two.
        assert_eq!(
            pairs("from $20 and then we all waited so very long to $90"),
            []
        );
        // Eight between: a pair. Seven: a pair, but not a close one.
        assert_eq!(
            pairs("from $20 and then we all waited so long to $90").len(),
            1
        );
        let c = one("from $20 and then we waited so long to $90");
        assert!(close(c.score, 0.95));
        assert_eq!(pairs("We started at $20. Then it went to $90."), []);
    }

    #[test]
    fn each_quantity_keeps_its_best_partner_then_the_nearest() {
        let found = pairs("It went from $10 to $20, not $30.");
        assert_eq!(found.len(), 2);
        assert_eq!(shown(&found[0]), ("$10", "$20"));
        assert_eq!(found[0].score, 1.0);
        assert_eq!(shown(&found[1]), ("$20", "$30"));
        // Equal scores: the nearer partner.
        let found = pairs("$10 or $20 or $30");
        assert_eq!(shown(&found[0]), ("$10", "$20"));
    }

    #[test]
    fn a_pair_spans_both_quantities_and_has_them_as_its_parts() {
        let mut t = transcript("The rough cut went from 90 minutes to 10 minutes.");
        let c = pairs_of(&t).remove(0);
        let range = |start, end| WordRange {
            start: WordIdx::new(start),
            end: WordIdx::new(end),
        };
        assert_eq!(c.kind, EventKind::FromTo);
        assert_eq!(c.anchors, range(5, 10));
        assert_eq!(c.parts, [range(5, 7), range(8, 10)]);
        assert_eq!((c.span.start.get(), c.span.end.get()), (2_000, 3_900));
        // The least certain word of either quantity counts; words between do not.
        t.words[7].confidence = Confidence::new(0.5).unwrap();
        assert_eq!(pairs_of(&t)[0].score, 1.0);
        t.words[9].confidence = Confidence::new(0.5).unwrap();
        assert_eq!(pairs_of(&t)[0].score, 0.5);
    }
}
